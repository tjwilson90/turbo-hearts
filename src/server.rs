use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{GameFeEvent, Games},
    hacks::UnboundedReceiver,
    lobby::{Lobby, LobbyEvent},
    types::{ChargingRules, GameId, Player},
};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Server {
    lobby: Lobby,
    games: Games,
}

impl Server {
    pub fn new(db: Database) -> Result<Self, CardsError> {
        let games = db.hydrate_games()?;
        Ok(Self {
            lobby: Lobby::new(games),
            games: Games::new(db),
        })
    }

    pub async fn ping_event_streams(&self) {
        self.lobby.ping().await;
        self.games.ping().await;
    }

    pub async fn subscribe_lobby(&self, player: Player) -> UnboundedReceiver<LobbyEvent> {
        self.lobby.subscribe(player).await
    }

    pub async fn new_game(&self, player: Player, rules: ChargingRules) -> GameId {
        self.lobby.new_game(player, rules).await
    }

    pub async fn join_game(
        &self,
        id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<HashSet<Player>, CardsError> {
        let players = self.lobby.join_game(id, player, rules).await?;
        if players.len() == 4 {
            self.games.start_game(id, &players)?;
        }
        Ok(players.keys().cloned().collect())
    }

    pub async fn leave_game(&self, id: GameId, player: Player) {
        self.lobby.leave_game(id, player).await
    }

    pub async fn subscribe_game(
        &self,
        id: GameId,
        player: Player,
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        self.games.subscribe(id, player).await
    }

    pub async fn pass_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.games.pass_cards(id, player, cards).await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.games.charge_cards(id, player, cards).await
    }

    pub async fn play_card(
        &self,
        id: GameId,
        player: Player,
        card: Card,
    ) -> Result<(), CardsError> {
        let complete = self.games.play_card(id, player, card).await?;
        if complete {
            self.lobby.remove_game(id).await;
        }
        Ok(())
    }
}
