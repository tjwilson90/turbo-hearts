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
use rusqlite::{Transaction, NO_PARAMS};
use std::collections::HashMap;
use tokio::task;

#[derive(Clone)]
pub struct Server {
    lobby: Lobby,
    games: Games,
}

impl Server {
    pub async fn new(db: Database) -> Result<Self, CardsError> {
        let partial_games = db.run_read_only(|tx| hydrate_games(&tx))?;
        let games = Games::new(db);
        for (id, participants) in &partial_games {
            start_bots(&games, *id, participants).await;
        }
        Ok(Self {
            lobby: Lobby::new(partial_games),
            games,
        })
    }

    pub async fn ping_event_streams(&self) {
        self.lobby.ping().await;
        self.games.ping().await;
    }

    pub async fn subscribe_lobby(&self, name: Name) -> UnboundedReceiver<LobbyEvent> {
        self.lobby.subscribe(name).await
    }

    pub async fn new_game(&self, name: Name, rules: ChargingRules) -> GameId {
        self.lobby.new_game(name, rules).await
    }

    pub async fn join_game(
        &self,
        id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<Vec<Player>, CardsError> {
        let participants = self.lobby.join_game(id, player, rules).await?;
        if participants.len() == 4 {
            self.games.start_game(id, &participants)?;
            start_bots(&self.games, id, &participants).await;
        }
        Ok(participants
            .into_iter()
            .map(|participant| participant.player)
            .collect())
    }

    pub async fn leave_game(&self, id: GameId, name: Name) {
        self.lobby.leave_game(id, name).await
    }

    pub async fn subscribe_game(
        &self,
        id: GameId,
        name: Name,
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        self.games.subscribe(id, name).await
    }

    pub async fn pass_cards(&self, id: GameId, name: Name, cards: Cards) -> Result<(), CardsError> {
        self.games.pass_cards(id, name, cards).await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        name: Name,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.games.charge_cards(id, name, cards).await
    }

    pub async fn play_card(&self, id: GameId, name: Name, card: Card) -> Result<(), CardsError> {
        let complete = self.games.play_card(id, name, card).await?;
        if complete {
            self.lobby.remove_game(id).await;
        }
        Ok(())
    }
}

async fn start_bots(games: &Games, id: GameId, participants: &[Participant]) {
    if participants.len() < 4 {
        return;
    }
    for participant in participants {
        if let Player::Bot { name, algorithm } = &participant.player {
            let bot = Bot::new(name.clone(), algorithm);
            task::spawn(bot.run(games.clone(), id));
        }
    }
}

fn hydrate_games(tx: &Transaction) -> Result<HashMap<GameId, Vec<Participant>>, CardsError> {
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
            let mut participants = Vec::new();
            participants.push(Participant {
                player: north,
                rules,
            });
            participants.push(Participant {
                player: east,
                rules,
            });
            participants.push(Participant {
                player: south,
                rules,
            });
            participants.push(Participant {
                player: west,
                rules,
            });
            games.insert(id, participants);
        }
    }
    Ok(games)
}
