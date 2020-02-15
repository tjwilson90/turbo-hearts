use crate::{
    cards::{Card, Cards, GamePhase},
    db::Database,
    error::CardsError,
    game::{persist_events, GameEvent},
    lobby::LobbyEvent,
    server::Server,
    types::{ChargingRules, GameId, Player},
};
use log::LevelFilter;
use r2d2_sqlite::SqliteConnectionManager;
use std::future::Future;
use tempfile::TempDir;

macro_rules! s {
    ($string:ident) => {
        stringify!($string).to_string()
    };
}

macro_rules! h {
    ($name:ident) => {
        crate::types::Player::Human { name: s!($name) }
    };
}

macro_rules! c {
    ($($cards:tt)*) => {
        stringify!($($cards)*).parse().unwrap()
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

macro_rules! matches {
    ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )?) => {
        match $expression {
            $( $pattern )|+ $( if $guard )? => true,
            _ => false
        }
    }
}

struct TestRunner {
    _temp_dir: TempDir,
    db: Database,
    server: Server,
}

impl TestRunner {
    fn new() -> Self {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .is_test(true)
            .try_init();
        let temp_dir = tempfile::tempdir().unwrap();
        let mut path = temp_dir.path().to_owned();
        path.push("test.db");
        let db = Database::new(SqliteConnectionManager::file(path)).unwrap();
        let server = Server::new(db.clone()).unwrap();
        Self {
            _temp_dir: temp_dir,
            db,
            server,
        }
    }

    async fn run<F, T>(&self, task: T) -> F::Output
    where
        T: FnOnce(Database, Server) -> F + Send + 'static,
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let db = self.db.clone();
        let server = self.server.clone();
        let result = tokio::spawn(async move { task(db, server).await }).await;
        match result {
            Ok(v) => v,
            Err(e) => std::panic::resume_unwind(e.into_panic()),
        }
    }
}

#[test]
fn test_card_display() {
    assert_eq!(Card::NineSpades.to_string(), "9S");
    assert_eq!(Card::ThreeDiamonds.to_string(), "3D");
    assert_eq!(Card::JackClubs.to_string(), "JC");
    assert_eq!(Card::AceHearts.to_string(), "AH");
}

#[test]
fn test_card_suit() {
    assert_eq!(Card::TwoClubs.suit().cards(), Cards::CLUBS);
    assert_eq!(Card::AceClubs.suit().cards(), Cards::CLUBS);
    assert_eq!(Card::TwoDiamonds.suit().cards(), Cards::DIAMONDS);
    assert_eq!(Card::AceDiamonds.suit().cards(), Cards::DIAMONDS);
    assert_eq!(Card::TwoHearts.suit().cards(), Cards::HEARTS);
    assert_eq!(Card::AceHearts.suit().cards(), Cards::HEARTS);
    assert_eq!(Card::TwoSpades.suit().cards(), Cards::SPADES);
    assert_eq!(Card::AceSpades.suit().cards(), Cards::SPADES);
}

#[test]
fn test_cards_display() {
    assert_eq!(
        format!(
            "{}",
            Card::NineSpades | Card::QueenSpades | Card::JackDiamonds
        ),
        "Q9S JD"
    );
}

#[test]
fn test_cards_max() {
    assert_eq!((Card::TwoClubs | Card::NineClubs).max(), Card::NineClubs);
    assert_eq!(
        (Card::FourHearts | Card::SevenDiamonds).max(),
        Card::FourHearts
    );
    assert_eq!((Card::AceSpades | Card::FiveSpades).max(), Card::AceSpades);
    assert_eq!(Cards::from(Card::FiveHearts).max(), Card::FiveHearts);
}

#[test]
fn test_cards_iter() {
    assert_eq!(
        (Card::QueenSpades | Card::AceHearts | Card::TenClubs | Card::JackDiamonds)
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            Card::QueenSpades,
            Card::AceHearts,
            Card::JackDiamonds,
            Card::TenClubs
        ]
    );
}

#[test]
fn test_cards_parse() {
    assert_eq!(Cards::from(Card::AceHearts), c!(AH))
}

#[tokio::test(threaded_scheduler)]
async fn test_join_unknown_game() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let id = GameId::new();
        let resp = server
            .join_game(id, h!(twilson), ChargingRules::Classic)
            .await;
        assert!(matches!(resp, Err(CardsError::UnknownGame(game)) if game == id));
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_lobby() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let mut twilson = server.subscribe_lobby(s!(twilson)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![s!(twilson)],
                games: map![],
            })
        );

        let mut carrino = server.subscribe_lobby(s!(carrino)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinLobby { name: s!(carrino) })
        );
        assert_eq!(
            carrino.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![s!(twilson), s!(carrino)],
                games: map![],
            })
        );

        drop(carrino);
        server.ping_event_streams().await;
        assert_eq!(twilson.recv().await, Some(LobbyEvent::Ping));
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveLobby { name: s!(carrino) })
        );

        let id = server.new_game("tslatcher", ChargingRules::Bridge).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::NewGame {
                id,
                name: s!(tslatcher),
            })
        );

        drop(twilson);
        server.ping_event_streams().await;
        let mut twilson = server.subscribe_lobby(s!(twilson)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![s!(twilson)],
                games: map![id => vec![h!(tslatcher)]],
            })
        );

        let players = server
            .join_game(id, h!(dcervelli), ChargingRules::Classic)
            .await?;
        assert_eq!(players, set![h!(tslatcher), h!(dcervelli)]);
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinGame {
                id,
                player: h!(dcervelli),
            })
        );

        server.leave_game(id, s!(tslatcher)).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveGame {
                id,
                name: s!(tslatcher),
            })
        );
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_new_game() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let id = server.new_game("twilson", ChargingRules::Classic).await;
        server
            .join_game(id, h!(tslatcher), ChargingRules::Classic)
            .await?;
        server
            .join_game(id, h!(dcervelli), ChargingRules::Classic)
            .await?;
        server
            .join_game(id, h!(carrino), ChargingRules::Classic)
            .await?;

        let mut twilson = server.subscribe_game(id, s!(twilson)).await?;
        match twilson.recv().await {
            Some(GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules: ChargingRules::Classic,
            }) => assert_eq!(
                set![north, east, south, west],
                set![h!(twilson), h!(tslatcher), h!(dcervelli), h!(carrino)]
            ),
            e => assert!(false, "Expected sit event, found {:?}", e),
        }
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_pass() -> Result<(), CardsError> {
    async fn test(db: Database, server: Server) -> Result<(), CardsError> {
        let id = GameId::new();
        db.run_with_retry(|tx| {
            persist_events(
                &tx,
                id,
                0,
                &[
                    GameEvent::Sit {
                        north: h!(twilson),
                        east: h!(dcervelli),
                        south: h!(tslatcher),
                        west: h!(carrino),
                        rules: ChargingRules::Classic,
                    },
                    GameEvent::Deal {
                        north: c!(A764S A96H AJD K863C),
                        east: c!(JT953S QT4H K93D ATC),
                        south: c!(2S 875H T542D QJ752C),
                        west: c!(KQ8S KJ32H Q876D 94C),
                    },
                ],
            )?;
            Ok(())
        })?;

        assert!(matches!(
            server.pass_cards(id, "twilson", c!(A73S)).await,
            Err(CardsError::NotYourCards(c)) if c == c!(3S)
        ));

        assert!(matches!(
            server.pass_cards(id, "twilson", c!(A7S A9H)).await,
            Err(CardsError::IllegalPassSize(c)) if c == c!(A7S A9H)
        ));

        assert!(matches!(
            server.charge_cards(id, "twilson", c!(AH)).await,
            Err(CardsError::IllegalAction(phase)) if phase == GamePhase::PassLeft
        ));

        assert!(matches!(
            server.play_card(id, "twilson", Card::SixClubs).await,
            Err(CardsError::IllegalAction(phase)) if phase == GamePhase::PassLeft
        ));

        server.pass_cards(id, "twilson", c!(K86C)).await?;

        assert!(matches!(
            server.pass_cards(id, "twilson", c!(A96H)).await,
            Err(CardsError::AlreadyPassed(c)) if c == c!(K86C)
        ));

        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_random_bot_game() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let id = server.new_game("fake", ChargingRules::Classic).await;
        server
            .join_game(
                id,
                Player::Bot {
                    name: s!(one),
                    algorithm: s!(random),
                },
                ChargingRules::BlindChain,
            )
            .await?;
        server.leave_game(id, s!(fake)).await;
        server
            .join_game(
                id,
                Player::Bot {
                    name: s!(two),
                    algorithm: s!(random),
                },
                ChargingRules::Classic,
            )
            .await?;
        server
            .join_game(
                id,
                Player::Bot {
                    name: s!(three),
                    algorithm: s!(random),
                },
                ChargingRules::Bridge,
            )
            .await?;
        server
            .join_game(
                id,
                Player::Bot {
                    name: s!(four),
                    algorithm: s!(random),
                },
                ChargingRules::Blind,
            )
            .await?;
        let mut rx = server.subscribe_game(id, s!(foo)).await?;
        let mut plays = 0;
        while let Some(event) = rx.recv().await {
            if let GameEvent::Play { .. } = event {
                plays += 1;
            }
        }
        assert_eq!(plays, 208);
        Ok(())
    }
    let runner = TestRunner::new();
    for _ in 0..30 {
        runner.run(test).await?;
    }
    Ok(())
}
