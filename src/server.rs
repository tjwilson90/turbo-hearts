use crate::{
    bot::Bot,
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{GameEvent, Games},
    lobby::{Lobby, LobbyEvent, LobbyGame},
    types::{GameId, Player, PlayerWithOptions, Seat, UserId},
};
use log::info;
use rand_distr::Gamma;
use rusqlite::{Transaction, NO_PARAMS};
use std::collections::{HashMap, HashSet};
use tokio::{stream::StreamExt, sync::mpsc::UnboundedReceiver, task, time, time::Duration};

#[derive(Clone)]
pub struct Server {
    pub bot_delay: Option<Gamma<f32>>,
    lobby: Lobby,
    games: Games,
}

impl Server {
    #[cfg(test)]
    pub fn with_fast_bots(db: Database) -> Result<Self, CardsError> {
        Server::new(db, None)
    }

    pub fn with_slow_bots(db: Database, bot_delay: Gamma<f32>) -> Result<Self, CardsError> {
        Self::new(db, Some(bot_delay))
    }

    fn new(db: Database, bot_delay: Option<Gamma<f32>>) -> Result<Self, CardsError> {
        let partial_games = db.run_blocking_read_only(|tx| hydrate_games(&tx))?;
        let server = Self {
            bot_delay,
            lobby: Lobby::new(partial_games.clone()),
            games: Games::new(db),
        };
        for (game_id, lobby) in &partial_games {
            server.start_bots(*game_id, &lobby.players);
        }
        Ok(server)
    }

    fn start_bots(&self, game_id: GameId, participants: &HashSet<PlayerWithOptions>) {
        if participants.len() < 4 {
            return;
        }
        for participant in participants {
            if let Player::Bot { user_id, strategy } = participant.player {
                info!(
                    "Starting bot {} with strategy {:?} in game {}",
                    user_id, strategy, game_id
                );
                let bot = Bot::new(user_id, strategy);
                task::spawn(bot.run(self.clone(), game_id));
            }
        }
    }

    pub fn start_background_pings(self) {
        tokio::task::spawn(async move {
            let mut stream = time::interval(Duration::from_secs(15));
            while let Some(_) = stream.next().await {
                self.ping_event_streams().await;
            }
        });
    }

    pub async fn ping_event_streams(&self) {
        self.lobby.ping().await;
        self.games.ping().await;
    }

    pub async fn subscribe_lobby(&self, user_id: UserId) -> UnboundedReceiver<LobbyEvent> {
        info!("User {} joined the lobby", user_id);
        self.lobby.subscribe(user_id).await
    }

    pub async fn new_game(&self, player: PlayerWithOptions, seed: Option<String>) -> GameId {
        let game_id = self.lobby.new_game(player, seed).await;
        info!("{:?} started game {}", player, game_id);
        game_id
    }

    pub async fn join_game(
        &self,
        game_id: GameId,
        player: PlayerWithOptions,
    ) -> Result<(), CardsError> {
        let game = match self.lobby.join_game(game_id, player.clone()).await {
            Ok(participants) => {
                info!("{:?} joined game {}", player, game_id);
                participants
            }
            Err(e) => {
                info!(
                    "{:?} failed to join game {} with error {}",
                    player, game_id, e
                );
                return Err(e);
            }
        };
        if game.players.len() == 4 {
            info!("starting game {}", game_id);
            self.games.start_game(game_id, &game)?;
            self.start_bots(game_id, &game.players);
        }
        Ok(())
    }

    pub async fn leave_game(&self, game_id: GameId, user_id: UserId) {
        info!("{:?} left game {}", user_id, game_id);
        self.lobby.leave_game(game_id, user_id).await
    }

    pub async fn lobby_chat(&self, user_id: UserId, message: String) {
        self.lobby.chat(user_id, message).await
    }

    pub async fn subscribe_game(
        &self,
        game_id: GameId,
        user_id: UserId,
    ) -> Result<UnboundedReceiver<GameEvent>, CardsError> {
        info!("User {} subscribed to game {}", user_id, game_id);
        self.games.subscribe(game_id, user_id).await
    }

    pub async fn pass_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self.games.pass_cards(game_id, user_id, cards).await;
        match &result {
            Ok(_) => info!(
                "User {} passed {} in game {} successfully",
                user_id, cards, game_id
            ),
            Err(e) => info!(
                "User {} failed to pass {} in game {} with error {}",
                user_id, cards, game_id, e
            ),
        }
        result
    }

    pub async fn charge_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self.games.charge_cards(game_id, user_id, cards).await;
        match &result {
            Ok(_) => info!(
                "User {} charged {} in game {} successfully",
                user_id, cards, game_id
            ),
            Err(e) => info!(
                "User {} failed to charge {} in game {} with error {}",
                user_id, cards, game_id, e
            ),
        }
        result
    }

    pub async fn play_card(
        &self,
        game_id: GameId,
        user_id: UserId,
        card: Card,
    ) -> Result<bool, CardsError> {
        let result = self.games.play_card(game_id, user_id, card).await;
        match &result {
            Ok(complete) => {
                info!(
                    "User {} played {} in game {} successfully",
                    user_id, card, game_id
                );
                if *complete {
                    self.lobby.remove_game(game_id).await;
                    info!("Removed completed game {} from the lobby", game_id);
                }
            }
            Err(e) => info!(
                "User {} failed to play {} in game {} with error {}",
                user_id, card, game_id, e
            ),
        }
        result
    }

    pub async fn claim(&self, game_id: GameId, user_id: UserId) -> Result<(), CardsError> {
        let result = self.games.claim(game_id, user_id).await;
        match &result {
            Ok(()) => info!(
                "User {} made a claim in game {} successfully",
                user_id, game_id
            ),
            Err(e) => info!(
                "User {} failed to claim in game {} with error {}",
                user_id, game_id, e
            ),
        }
        result
    }

    pub async fn accept_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<(), CardsError> {
        let result = self.games.accept_claim(game_id, user_id, claimer).await;
        match &result {
            Ok(()) => info!(
                "User {} accepted the claim from {} in game {} successfully",
                user_id, claimer, game_id
            ),
            Err(e) => info!(
                "User {} failed to accept the claim from {} in game {} with error {}",
                user_id, claimer, game_id, e
            ),
        }
        result
    }

    pub async fn reject_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<(), CardsError> {
        let result = self.games.reject_claim(game_id, user_id, claimer).await;
        match &result {
            Ok(()) => info!(
                "User {} rejected the claim from {} in game {} successfully",
                user_id, claimer, game_id
            ),
            Err(e) => info!(
                "User {} failed to reject the claim from {} in game {} with error {}",
                user_id, claimer, game_id, e
            ),
        }
        result
    }

    pub async fn game_chat(
        &self,
        game_id: GameId,
        user_id: UserId,
        message: String,
    ) -> Result<(), CardsError> {
        self.games.chat(game_id, user_id, message).await
    }
}

fn hydrate_games(tx: &Transaction) -> Result<HashMap<GameId, LobbyGame>, CardsError> {
    let mut stmt = tx.prepare(
        "SELECT game_id, event, timestamp FROM event
            WHERE event_id = 0 AND game_id NOT IN (SELECT game_id FROM game)",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut games = HashMap::new();
    while let Some(row) = rows.next()? {
        let game_id = row.get(0)?;
        let timestamp = row.get(2)?;
        if let GameEvent::Sit {
            north,
            east,
            south,
            west,
            rules,
            created_time,
            created_by,
            seed,
        } = serde_json::from_str(&row.get::<_, String>(1)?)?
        {
            let mut participants = HashSet::new();
            participants.insert(PlayerWithOptions {
                player: north,
                rules,
                seat: None,
            });
            participants.insert(PlayerWithOptions {
                player: east,
                rules,
                seat: None,
            });
            participants.insert(PlayerWithOptions {
                player: south,
                rules,
                seat: None,
            });
            participants.insert(PlayerWithOptions {
                player: west,
                rules,
                seat: None,
            });
            games.insert(
                game_id,
                LobbyGame {
                    players: participants,
                    seed,
                    created_time,
                    created_by,
                    last_updated_time: timestamp,
                    last_updated_by: north.user_id(),
                },
            );
        }
    }
    Ok(games)
}
