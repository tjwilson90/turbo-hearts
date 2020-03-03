use crate::{
    error::CardsError,
    game::id::GameId,
    lobby::event::LobbyEvent,
    types::{PlayerWithOptions, Seed},
    user::UserId,
};
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::SystemTime,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub mod endpoints;
pub mod event;

#[derive(Clone)]
pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LobbyGame {
    pub players: HashSet<PlayerWithOptions>,
    pub seed: Seed,
    pub created_time: i64,
    pub created_by: UserId,
    pub last_updated_time: i64,
    pub last_updated_by: UserId,
}

impl LobbyGame {
    fn redact(&self) -> Self {
        LobbyGame {
            players: self.players.clone(),
            seed: self.seed.redact(),
            created_time: self.created_time,
            created_by: self.created_by,
            last_updated_time: self.last_updated_time,
            last_updated_by: self.last_updated_by,
        }
    }
}

struct Inner {
    subscribers: HashMap<UserId, Vec<UnboundedSender<LobbyEvent>>>,
    games: HashMap<GameId, LobbyGame>,
}

impl Lobby {
    pub fn new(games: HashMap<GameId, LobbyGame>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                subscribers: HashMap::new(),
                games,
            })),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Ping);
    }

    pub async fn subscribe(&self, user_id: UserId) -> UnboundedReceiver<LobbyEvent> {
        let (tx, rx) = unbounded_channel();
        let mut inner = self.inner.lock().await;
        if !inner.subscribers.contains_key(&user_id) {
            inner.broadcast(LobbyEvent::JoinLobby { user_id });
        }
        inner
            .subscribers
            .entry(user_id)
            .or_insert(Vec::new())
            .push(tx.clone());
        tx.send(LobbyEvent::LobbyState {
            subscribers: inner.subscribers.keys().cloned().collect(),
            games: inner
                .games
                .iter()
                .map(|(game_id, lobby)| (*game_id, lobby.redact()))
                .collect(),
        })
        .unwrap();
        rx
    }

    pub async fn new_game(&self, player: PlayerWithOptions, seed: Option<String>) -> GameId {
        let game_id = GameId::new();
        let user_id = player.player.user_id();
        let mut inner = self.inner.lock().await;
        let mut participants = HashSet::new();
        participants.insert(player);
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let game = LobbyGame {
            players: participants,
            seed: seed.map_or_else(|| Seed::random(), |value| Seed::Chosen { value }),
            created_time: timestamp,
            created_by: user_id,
            last_updated_time: timestamp,
            last_updated_by: user_id,
        };
        let redacted = game.redact();
        inner.games.insert(game_id, game);
        inner.broadcast(LobbyEvent::NewGame {
            game_id,
            game: redacted,
        });
        game_id
    }

    pub async fn join_game(
        &self,
        game_id: GameId,
        player: PlayerWithOptions,
    ) -> Result<LobbyGame, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(game) = inner.games.get_mut(&game_id) {
            if game.players.len() == 4 {
                return Err(CardsError::GameHasStarted(game_id));
            }
            game.players.insert(player.clone());
            game.last_updated_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            let game = game.clone();
            inner.broadcast(LobbyEvent::JoinGame { game_id, player });
            Ok(game)
        } else {
            Err(CardsError::UnknownGame(game_id))
        }
    }

    pub async fn leave_game(&self, game_id: GameId, user_id: UserId) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(game_id) {
            let participants = &mut entry.get_mut().players;
            let count = participants.len();
            participants.retain(|participant| participant.player.user_id() != user_id);
            if participants.len() < count {
                if participants.is_empty() {
                    entry.remove();
                }
                inner.broadcast(LobbyEvent::LeaveGame { game_id, user_id });
            }
        }
    }

    pub async fn remove_game(&self, game_id: GameId) {
        let mut inner = self.inner.lock().await;
        if inner.games.remove(&game_id).is_some() {
            inner.broadcast(LobbyEvent::FinishGame { game_id });
        }
    }

    pub async fn chat(&self, user_id: UserId, message: String) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Chat { user_id, message });
    }
}

impl Inner {
    fn broadcast(&mut self, event: LobbyEvent) {
        let mut events = VecDeque::new();
        events.push_back(event);
        while let Some(event) = events.pop_front() {
            self.subscribers.retain(|user_id, txs| {
                txs.retain(|tx| tx.send(event.clone()).is_ok());
                if txs.is_empty() {
                    events.push_back(LobbyEvent::LeaveLobby { user_id: *user_id });
                    false
                } else {
                    true
                }
            });
        }
    }
}
