use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{GameEvent, Games},
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
        let complete = self.games.play_card(id, player, card).await?;
        if complete {
            self.lobby.remove_game(id).await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2_sqlite::SqliteConnectionManager;
    use std::future::Future;

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

    async fn run<F, T>(task: T) -> F::Output
    where
        T: FnOnce(Database) -> F + Send + 'static,
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let _ = env_logger::builder().is_test(true).try_init();
        tokio::spawn(async move {
            let dir = tempfile::tempdir().unwrap();
            let mut path = dir.path().to_owned();
            path.push("test.db");
            let db = Database::new(SqliteConnectionManager::file(path)).unwrap();
            task(db).await
        })
        .await
        .unwrap()
    }

    #[tokio::test(threaded_scheduler)]
    async fn join_unknown_game() -> Result<(), CardsError> {
        run(async move |db| {
            let server = Server::new(db)?;
            let resp = server
                .join_game(GameId::new(), p!(twilson), ChargingRules::Classic)
                .await;
            assert!(matches!(resp, Err(CardsError::UnknownGame(_))));
            Ok(())
        })
        .await
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_lobby() -> Result<(), CardsError> {
        run(async move |db| {
            let server = Server::new(db)?;

            let mut twilson = server.subscribe_lobby(p!(twilson)).await;
            assert_eq!(
                twilson.recv().await,
                Some(LobbyEvent::LobbyState {
                    subscribers: set![p!(twilson)],
                    games: map![],
                })
            );

            let mut carrino = server.subscribe_lobby(p!(carrino)).await;
            assert_eq!(
                twilson.recv().await,
                Some(LobbyEvent::JoinLobby {
                    player: p!(carrino),
                })
            );
            assert_eq!(
                carrino.recv().await,
                Some(LobbyEvent::LobbyState {
                    subscribers: set![p!(twilson), p!(carrino)],
                    games: map![],
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
                })
            );

            drop(twilson);
            server.ping_event_streams().await;
            let mut twilson = server.subscribe_lobby(p!(twilson)).await;
            assert_eq!(
                twilson.recv().await,
                Some(LobbyEvent::LobbyState {
                    subscribers: set![p!(twilson)],
                    games: map![id => set![p!(tslatcher)]],
                })
            );

            let players = server
                .join_game(id, p!(dcervelli), ChargingRules::Classic)
                .await?;
            assert_eq!(players, set![p!(tslatcher), p!(dcervelli)]);
            assert_eq!(
                twilson.recv().await,
                Some(LobbyEvent::JoinGame {
                    id,
                    player: p!(dcervelli),
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
        })
        .await
    }

    #[tokio::test(threaded_scheduler)]
    async fn test_game() -> Result<(), CardsError> {
        run(async move |db| {
            let server = Server::new(db)?;
            let id = server.new_game(p!(twilson), ChargingRules::Classic).await;
            server
                .join_game(id, p!(tslatcher), ChargingRules::Classic)
                .await?;
            server
                .join_game(id, p!(dcervelli), ChargingRules::Classic)
                .await?;
            server
                .join_game(id, p!(carrino), ChargingRules::Classic)
                .await?;

            let mut twilson = server.subscribe_game(id, p!(twilson)).await?;
            match twilson.recv().await {
                Some(GameEvent::Sit {
                    north,
                    east,
                    south,
                    west,
                    rules: ChargingRules::Classic,
                }) => assert_eq!(
                    set![north, east, south, west],
                    set![p!(twilson), p!(tslatcher), p!(dcervelli), p!(carrino)]
                ),
                e => assert!(false, "Expected sit event, found {:?}", e),
            }
            Ok(())
        })
        .await
    }
}
