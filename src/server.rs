use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{GameEvent, Games},
    hacks::UnboundedReceiver,
    lobby::{Lobby, LobbyEvent},
    types::{ChargingRules, GameId, Player},
};
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Server {
    lobby: Lobby,
    games: Games,
}

impl Server {
    pub fn new(manager: SqliteConnectionManager) -> Self {
        Self {
            lobby: Lobby::new(),
            games: Games::new(Database::new(manager)),
        }
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
    ) -> Result<HashMap<Player, ChargingRules>, CardsError> {
        let players = self.lobby.join_game(id, player, rules).await?;
        if players.len() == 4 {
            self.games.start_game(id, &players)?;
        }
        Ok(players)
    }

    pub async fn leave_game(&self, id: GameId, player: Player) {
        self.lobby.leave_game(id, player).await
    }

    pub async fn subscribe_game(
        &self,
        id: GameId,
        player: Player,
    ) -> Result<UnboundedReceiver<GameEvent>, CardsError> {
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
        self.games.play_card(id, player, card).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! p {
        ($p:ident) => {
            stringify!($p).to_string()
        };
    }

    macro_rules! set {
        ($($x:expr),*) => (
            vec![$($x),*].into_iter().collect::<std::collections::HashSet<_>>()
        );
    }

    macro_rules! map {
        ($($x:expr => $y:expr),*) => (
            vec![$(($x, $y)),*].into_iter().collect::<std::collections::HashMap<_, _>>()
        );
    }

    #[tokio::test]
    async fn join_unknown_game() {
        let server = Server::new(SqliteConnectionManager::memory());
        let resp = server
            .join_game(GameId::new(), p!(twilson), ChargingRules::Classic)
            .await;
        assert!(matches!(resp, Err(CardsError::UnknownGame(_))));
    }

    #[tokio::test]
    async fn test_lobby() -> Result<(), CardsError> {
        let server = Server::new(SqliteConnectionManager::memory());

        let mut twilson = server.subscribe_lobby(p!(twilson)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![p!(twilson)],
                games: HashMap::new(),
            })
        );

        let mut carrino = server.subscribe_lobby(p!(carrino)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::Subscribe {
                player: p!(carrino),
            })
        );
        assert_eq!(
            carrino.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![p!(twilson), p!(carrino)],
                games: HashMap::new(),
            })
        );

        drop(carrino);
        server.ping_event_streams().await;
        assert_eq!(twilson.recv().await, Some(LobbyEvent::Ping));
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveLobby {
                player: p!(carrino),
            })
        );

        let id = server.new_game(p!(tslatcher), ChargingRules::Bridge).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::NewGame {
                id,
                player: p!(tslatcher),
                rules: ChargingRules::Bridge
            })
        );

        drop(twilson);
        server.ping_event_streams().await;
        let mut twilson = server.subscribe_lobby(p!(twilson)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![p!(twilson)],
                games: map![id => map![p!(tslatcher) => ChargingRules::Bridge]],
            })
        );

        let players = server
            .join_game(id, p!(dcervelli), ChargingRules::Classic)
            .await?;
        assert_eq!(
            players,
            map![
                p!(tslatcher) => ChargingRules::Bridge,
                p!(dcervelli) => ChargingRules::Classic
            ]
        );
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinGame {
                id,
                player: p!(dcervelli),
                rules: ChargingRules::Classic
            })
        );

        server.leave_game(id, p!(tslatcher)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveGame {
                id,
                player: p!(tslatcher),
            })
        );
        Ok(())
    }
}
