use crate::{db::Database, lobby::event::LobbyEvent, util};
use log::info;
use rusqlite::{OptionalExtension, ToSql, Transaction, NO_PARAMS};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use turbo_hearts_api::{CardsError, GameId, Player, PlayerWithOptions, Seat, Seed, UserId};

pub mod endpoints;
pub mod event;

//#[derive(Clone)]
pub struct Lobby {
    db: &'static Database,
    inner: Mutex<Inner>,
}

impl Lobby {
    pub fn new(db: &'static Database) -> Result<Self, CardsError> {
        Ok(Self {
            db,
            inner: Mutex::new(Inner {
                subscribers: Vec::new(),
            }),
        })
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Ping);
    }

    pub async fn delete_stale_games(&self) -> Result<(), CardsError> {
        self.db.run_with_retry(|tx| {
            let now = util::timestamp();
            let rows = tx.execute(
                "DELETE FROM game WHERE (started_time IS NULL AND last_updated_time < ?)
                    OR (completed_time IS NULL AND last_updated_time < ?)",
                &[now - 24 * 60 * 60 * 1000, now - 21 * 24 * 60 * 60 * 1000],
            )?;
            if rows > 0 {
                info!("Deleted {} stale game(s)", rows);
            }
            let rows = tx.execute(
                "DELETE FROM game_player WHERE game_id NOT IN (SELECT game_id FROM game)",
                NO_PARAMS,
            )?;
            if rows > 0 {
                info!("Deleted {} stale game player(s)", rows);
            }
            Ok(())
        })?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        user_id: UserId,
    ) -> Result<UnboundedReceiver<LobbyEvent>, CardsError> {
        let (tx, rx) = mpsc::unbounded_channel();
        let (chat, games) = self.db.run_read_only(|tx| {
            let chat = load_recent_chat(&tx)?;
            let games = load_games(&tx)?;
            Ok((chat, games))
        })?;
        let mut inner = self.inner.lock().await;
        let mut subscribers = inner
            .subscribers
            .iter()
            .map(|(user_id, _)| *user_id)
            .collect::<HashSet<_>>();
        if subscribers.insert(user_id) {
            inner.broadcast(LobbyEvent::JoinLobby { user_id });
        }
        inner.subscribers.push((user_id, tx.clone()));
        tx.send(LobbyEvent::LobbyState {
            subscribers,
            chat,
            games,
        })
        .unwrap();
        info!("subscribe: user_id={}", user_id);
        Ok(rx)
    }

    pub async fn new_game(
        &self,
        player: PlayerWithOptions,
        seed: Option<String>,
    ) -> Result<GameId, CardsError> {
        let game_id = GameId::new();
        let user_id = player.player.user_id();
        let timestamp = util::timestamp();
        let seed = seed.map_or_else(|| Seed::random(), |value| Seed::Chosen { value });
        self.db.run_with_retry(|tx| {
            tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO game (game_id, seed, created_time, created_by,
                    last_updated_time, last_updated_by) VALUES (?, ?, ?, ?, ?, ?)",
                &[&game_id, &seed, &timestamp, &user_id, &timestamp, &user_id],
            )?;
            insert_player(&tx, game_id, &player)?;
            Ok(())
        })?;
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::NewGame {
            game_id,
            player,
            seed: seed.redact(),
        });
        info!(
            "new_game: game_id={}, player={:?}, seed={:?}",
            game_id, player, seed
        );
        Ok(game_id)
    }

    pub async fn join_game(
        &self,
        game_id: GameId,
        player: PlayerWithOptions,
    ) -> Result<(), CardsError> {
        let joined = self.db.run_with_retry(|tx| {
            validate_game_unstarted(&tx, game_id)?;
            if insert_player(&tx, game_id, &player)? {
                tx.execute::<&[&dyn ToSql]>(
                    "UPDATE game SET last_updated_time = ?, last_updated_by = ? WHERE game_id = ?",
                    &[&util::timestamp(), &player.player.user_id(), &game_id],
                )?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        if joined {
            let mut inner = self.inner.lock().await;
            inner.broadcast(LobbyEvent::JoinGame { game_id, player });
        }
        info!("join_game: game_id={}, player={:?}", game_id, player);
        Ok(())
    }

    pub async fn start_game(
        &self,
        game_id: GameId,
    ) -> Result<([PlayerWithOptions; 4], Seed), CardsError> {
        let (players, seed) = self.db.run_with_retry(|tx| {
            let mut stmt = tx.prepare_cached(
                "SELECT user_id, strategy, rules, seat FROM game_player
                    WHERE game_id = ? ORDER BY random() LIMIT 4",
            )?;
            let mut rows = stmt.query(&[game_id])?;
            let mut players = Vec::with_capacity(4);
            while let Some(row) = rows.next()? {
                let user_id = row.get(0)?;
                let player = if let Some(strategy) = row.get(1)? {
                    Player::Bot { user_id, strategy }
                } else {
                    Player::Human { user_id }
                };
                players.push(PlayerWithOptions {
                    player,
                    rules: row.get(2)?,
                    seat: row.get(3)?,
                });
            }
            if players.len() < 4 {
                return Err(CardsError::NotEnoughPlayers);
            }
            for &seat in &Seat::VALUES {
                if let Some(idx) = players.iter().position(|p| p.seat == Some(seat)) {
                    players.swap(idx, seat.idx());
                }
            }
            let seed = tx.query_row(
                "SELECT seed FROM game WHERE game_id = ?",
                &[game_id],
                |row| Ok(row.get::<_, Seed>(0)?),
            )?;
            Ok((players, seed))
        })?;
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::StartGame {
            game_id,
            north: players[0].player,
            east: players[1].player,
            south: players[2].player,
            west: players[3].player,
        });
        info!("start_game: game_id={}", game_id);
        Ok(([players[0], players[1], players[2], players[3]], seed))
    }

    pub async fn leave_game(&self, game_id: GameId, user_id: UserId) -> Result<(), CardsError> {
        let player = self.db.run_with_retry(|tx| {
            validate_game_unstarted(&tx, game_id)?;
            let player = remove_player(&tx, game_id, user_id)?;
            if player.is_some() {
                tx.execute::<&[&dyn ToSql]>(
                    "UPDATE game SET last_updated_time = ?, last_updated_by = ? WHERE game_id = ?",
                    &[&util::timestamp(), &user_id, &game_id],
                )?;
            }
            Ok(player)
        })?;
        if let Some(player) = player {
            let mut inner = self.inner.lock().await;
            inner.broadcast(LobbyEvent::LeaveGame { game_id, player });
        }
        info!("leave_game: game_id={}, user_id={}", game_id, user_id);
        Ok(())
    }

    pub async fn finish_game(&self, game_id: GameId) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::FinishGame { game_id });
        info!("finish_game: game_id={}", game_id);
    }

    pub async fn chat(&self, user_id: UserId, message: String) -> Result<(), CardsError> {
        self.db.run_with_retry(|tx| {
            tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO lobby_chat (timestamp, user_id, message) VALUES (?, ?, ?)",
                &[&util::timestamp(), &user_id, &message],
            )?;
            Ok(())
        })?;
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Chat { user_id, message });
        info!("chat: user_id={}", user_id);
        Ok(())
    }
}

struct Inner {
    subscribers: Vec<(UserId, UnboundedSender<LobbyEvent>)>,
}

impl Inner {
    fn broadcast(&mut self, event: LobbyEvent) {
        let mut disconnects = HashSet::new();
        self.subscribers.retain(|(user_id, tx)| {
            if tx.send(event.clone()).is_ok() {
                true
            } else {
                disconnects.insert(*user_id);
                false
            }
        });
        if !disconnects.is_empty() {
            for (user_id, _) in &self.subscribers {
                disconnects.remove(user_id);
            }
            for user_id in disconnects {
                self.broadcast(LobbyEvent::LeaveLobby { user_id });
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LobbyGame {
    pub players: HashSet<PlayerWithOptions>,
    pub seed: Seed,
    pub created_time: i64,
    pub created_by: UserId,
    pub last_updated_time: i64,
    pub last_updated_by: UserId,
    pub started_time: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LobbyChat {
    pub timestamp: i64,
    pub user_id: UserId,
    pub message: String,
}

fn load_recent_chat(tx: &Transaction) -> Result<Vec<LobbyChat>, CardsError> {
    let mut stmt = tx.prepare_cached(
        "SELECT timestamp, user_id, message FROM lobby_chat ORDER BY timestamp DESC LIMIT 500",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut messages = Vec::with_capacity(500);
    while let Some(row) = rows.next()? {
        messages.push(LobbyChat {
            timestamp: row.get(0)?,
            user_id: row.get(1)?,
            message: row.get(2)?,
        });
    }
    messages.reverse();
    Ok(messages)
}

fn load_games(tx: &Transaction) -> Result<HashMap<GameId, LobbyGame>, CardsError> {
    let mut games = HashMap::new();
    let mut stmt = tx.prepare_cached(
        "SELECT game_id, seed, created_time, created_by,
                last_updated_time, last_updated_by, started_time
                FROM game WHERE completed_time IS NULL AND last_updated_time > ?",
    )?;
    let mut rows = stmt.query(&[util::timestamp() - 8 * 60 * 60 * 1000])?;
    while let Some(row) = rows.next()? {
        games.insert(
            row.get(0)?,
            LobbyGame {
                players: HashSet::new(),
                seed: row.get::<_, Seed>(1)?.redact(),
                created_time: row.get(2)?,
                created_by: row.get(3)?,
                last_updated_time: row.get(4)?,
                last_updated_by: row.get(5)?,
                started_time: row.get(6)?,
            },
        );
    }
    let mut stmt = tx.prepare_cached(
        "SELECT gp.game_id, gp.user_id, gp.strategy, gp.rules, gp.seat
            FROM game_player gp, game g
            WHERE gp.game_id = g.game_id AND g.completed_time IS NULL",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;
    while let Some(row) = rows.next()? {
        if let Some(game) = games.get_mut(&row.get(0)?) {
            let user_id = row.get(1)?;
            let player = if let Some(strategy) = row.get(2)? {
                Player::Bot { user_id, strategy }
            } else {
                Player::Human { user_id }
            };
            game.players.insert(PlayerWithOptions {
                player,
                rules: row.get(3)?,
                seat: row.get(4)?,
            });
        }
    }
    games.retain(|_, game| !game.players.is_empty());
    Ok(games)
}

fn validate_game_unstarted(tx: &Transaction, game_id: GameId) -> Result<(), CardsError> {
    let started = tx
        .query_row(
            "SELECT started_time FROM game WHERE game_id = ?",
            &[game_id],
            |row| Ok(row.get::<_, Option<i64>>(0)?.is_some()),
        )
        .optional()?;
    match started {
        None => Err(CardsError::UnknownGame(game_id)),
        Some(true) => Err(CardsError::GameHasStarted(game_id)),
        _ => Ok(()),
    }
}

fn insert_player(
    tx: &Transaction,
    game_id: GameId,
    player: &PlayerWithOptions,
) -> Result<bool, CardsError> {
    let rows = tx.execute::<&[&dyn ToSql]>(
        "INSERT OR IGNORE INTO game_player (game_id, user_id, strategy, rules, seat)
            VALUES (?, ?, ?, ?, ?)",
        &[
            &game_id,
            &player.player.user_id(),
            &player.player.strategy(),
            &player.rules,
            &player.seat,
        ],
    )?;
    Ok(rows > 0)
}

fn remove_player(
    tx: &Transaction,
    game_id: GameId,
    user_id: UserId,
) -> Result<Option<Player>, CardsError> {
    let player = tx
        .query_row::<_, &[&dyn ToSql], _>(
            "SELECT strategy FROM game_player WHERE game_id = ? AND user_id = ?",
            &[&game_id, &user_id],
            |row| {
                Ok(match row.get(0)? {
                    Some(strategy) => Player::Bot { user_id, strategy },
                    None => Player::Human { user_id },
                })
            },
        )
        .optional()?;
    tx.execute::<&[&dyn ToSql]>(
        "DELETE FROM game_player WHERE game_id = ? AND user_id = ?",
        &[&game_id, &user_id],
    )?;
    Ok(player)
}
