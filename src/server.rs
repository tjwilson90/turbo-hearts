use crate::bot::Bot;
use crate::game::GameDbEvent;
use crate::types::{Name, Participant};
use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{GameFeEvent, Games},
    hacks::UnboundedReceiver,
    lobby::{Lobby, LobbyEvent},
    types::{ChargingRules, GameId, Player},
};
use log::info;
use rusqlite::{Transaction, NO_PARAMS};
use std::collections::{HashMap, HashSet};
use tokio::task;

#[derive(Clone)]
pub struct Server {
    lobby: Lobby,
    games: Games,
}

impl Server {
    pub async fn new(db: Database) -> Result<Self, CardsError> {
        let partial_games = db.run_read_only(|tx| hydrate_games(&tx))?;
        let server = Self {
            lobby: Lobby::new(partial_games.clone()),
            games: Games::new(db),
        };
        for (id, participants) in &partial_games {
            server.start_bots(*id, participants).await;
        }
        Ok(server)
    }

    async fn start_bots(&self, id: GameId, participants: &HashSet<Participant>) {
        if participants.len() < 4 {
            return;
        }
        for participant in participants {
            if let Player::Bot { name, algorithm } = &participant.player {
                info!("Starting {} with algorithm {}", name, algorithm);
                let bot = Bot::new(name.clone(), algorithm);
                task::spawn(bot.run(self.clone(), id));
            }
        }
    }

    pub async fn ping_event_streams(&self) {
        self.lobby.ping().await;
        self.games.ping().await;
    }

    pub async fn subscribe_lobby(&self, name: Name) -> UnboundedReceiver<LobbyEvent> {
        info!("{} joined the lobby", name);
        self.lobby.subscribe(name).await
    }

    pub async fn new_game(&self, name: Name, rules: ChargingRules) -> GameId {
        let id = self.lobby.new_game(name.clone(), rules).await;
        info!("{} started game {}", name, id);
        id
    }

    pub async fn join_game(
        &self,
        id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<HashSet<Player>, CardsError> {
        let participants = match self.lobby.join_game(id, player.clone(), rules).await {
            Ok(participants) => {
                info!("{:?} joined game {}", player, id);
                participants
            }
            Err(e) => {
                info!("{:?} failed to join game {} with error {}", player, id, e);
                return Err(e);
            }
        };
        if participants.len() == 4 {
            info!("starting game {}", id);
            self.games.start_game(id, &participants)?;
            self.start_bots(id, &participants).await;
        }
        Ok(participants
            .into_iter()
            .map(|participant| participant.player)
            .collect())
    }

    pub async fn leave_game(&self, id: GameId, name: Name) {
        info!("{} left game {}", name, id);
        self.lobby.leave_game(id, name).await
    }

    pub async fn subscribe_game(
        &self,
        id: GameId,
        name: Name,
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        info!("{} subscribed to game {}", name, id);
        self.games.subscribe(id, name).await
    }

    pub async fn pass_cards(&self, id: GameId, name: Name, cards: Cards) -> Result<(), CardsError> {
        let result = self.games.pass_cards(id, &name, cards).await;
        match &result {
            Ok(_) => info!("{} passed {} in game {} successfully", name, cards, id),
            Err(e) => info!(
                "{} failed to pass {} in game {} with error {}",
                name, cards, id, e
            ),
        }
        result
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        name: Name,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self.games.charge_cards(id, &name, cards).await;
        match &result {
            Ok(_) => info!("{} charged {} in game {} successfully", name, cards, id),
            Err(e) => info!(
                "{} failed to charge {} in game {} with error {}",
                name, cards, id, e
            ),
        }
        result
    }

    pub async fn play_card(&self, id: GameId, name: Name, card: Card) -> Result<bool, CardsError> {
        let result = self.games.play_card(id, &name, card).await;
        match &result {
            Ok(complete) => {
                info!("{} played {} in game {} successfully", name, card, id);
                if *complete {
                    self.lobby.remove_game(id).await;
                    info!("Removed completed game {} from the lobby", id);
                }
            }
            Err(e) => info!(
                "{} failed to play {} in game {} with error {}",
                name, card, id, e
            ),
        }
        result
    }
}

fn hydrate_games(tx: &Transaction) -> Result<HashMap<GameId, HashSet<Participant>>, CardsError> {
    let mut stmt = tx.prepare(
        "SELECT game_id, event FROM event
            WHERE event_id = 0 AND game_id NOT IN (SELECT id FROM game)",
    )?;
    let mut rows = stmt.query(NO_PARAMS)?;
    let mut games = HashMap::new();
    while let Some(row) = rows.next()? {
        let id = row.get(0)?;
        if let GameDbEvent::Sit {
            north,
            east,
            south,
            west,
            rules,
        } = serde_json::from_str(&row.get::<_, String>(1)?)?
        {
            let mut participants = HashSet::new();
            participants.insert(Participant {
                player: north,
                rules,
            });
            participants.insert(Participant {
                player: east,
                rules,
            });
            participants.insert(Participant {
                player: south,
                rules,
            });
            participants.insert(Participant {
                player: west,
                rules,
            });
            games.insert(id, participants);
        }
    }
    Ok(games)
}
