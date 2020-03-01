use crate::{
    cards::{Card, Cards, GamePhase, PassDirection},
    db::Database,
    error::CardsError,
    game::{persist_events, GameEvent},
    lobby::LobbyEvent,
    server::Server,
    types::{ChargingRules, GameId, Player, UserId},
};
use log::LevelFilter;
use once_cell::sync::Lazy;
use r2d2_sqlite::SqliteConnectionManager;
use std::future::Future;
use tempfile::TempDir;

macro_rules! h {
    ($user_id:expr) => {
        crate::types::Player::Human { user_id: $user_id }
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

static TWILSON: Lazy<UserId> = Lazy::new(|| UserId::new());
static TSLATCHER: Lazy<UserId> = Lazy::new(|| UserId::new());
static CARRINO: Lazy<UserId> = Lazy::new(|| UserId::new());
static DCERVELLI: Lazy<UserId> = Lazy::new(|| UserId::new());

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
        let server = Server::with_fast_bots(db.clone()).unwrap();
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
        let game_id = GameId::new();
        let resp = server
            .join_game(game_id, h!(*TWILSON), ChargingRules::Classic)
            .await;
        assert!(matches!(resp, Err(CardsError::UnknownGame(id)) if id == game_id));
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_lobby() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let mut twilson = server.subscribe_lobby(*TWILSON).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON],
                games: map![],
            })
        );

        let mut carrino = server.subscribe_lobby(*CARRINO).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinLobby { user_id: *CARRINO })
        );
        assert_eq!(
            carrino.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON, *CARRINO],
                games: map![],
            })
        );

        drop(carrino);
        server.ping_event_streams().await;
        assert_eq!(twilson.recv().await, Some(LobbyEvent::Ping));
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveLobby { user_id: *CARRINO })
        );

        let game_id = server.new_game(*TSLATCHER, ChargingRules::Bridge).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::NewGame {
                game_id,
                user_id: *TSLATCHER,
            })
        );

        drop(twilson);
        server.ping_event_streams().await;
        let mut twilson = server.subscribe_lobby(*TWILSON).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON],
                games: map![game_id => vec![h!(*TSLATCHER)]],
            })
        );

        let players = server
            .join_game(game_id, h!(*DCERVELLI), ChargingRules::Classic)
            .await?;
        assert_eq!(players, set![h!(*TSLATCHER), h!(*DCERVELLI)]);
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinGame {
                game_id,
                player: h!(*DCERVELLI),
            })
        );

        server.leave_game(game_id, *TSLATCHER).await;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveGame {
                game_id,
                user_id: *TSLATCHER,
            })
        );
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_new_game() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let game_id = server.new_game(*TWILSON, ChargingRules::Classic).await;
        server
            .join_game(game_id, h!(*TSLATCHER), ChargingRules::Classic)
            .await?;
        server
            .join_game(game_id, h!(*DCERVELLI), ChargingRules::Classic)
            .await?;
        server
            .join_game(game_id, h!(*CARRINO), ChargingRules::Classic)
            .await?;

        let mut twilson = server.subscribe_game(game_id, *TWILSON).await?;
        match twilson.recv().await {
            Some(GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules: ChargingRules::Classic,
                created_at: _,
            }) => assert_eq!(
                set![north, east, south, west],
                set![h!(*TWILSON), h!(*TSLATCHER), h!(*DCERVELLI), h!(*CARRINO)]
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
        let game_id = GameId::new();
        db.run_with_retry(|tx| {
            persist_events(
                &tx,
                game_id,
                0,
                &[
                    GameEvent::Sit {
                        north: h!(*TWILSON),
                        east: h!(*DCERVELLI),
                        south: h!(*TSLATCHER),
                        west: h!(*CARRINO),
                        rules: ChargingRules::Classic,
                        created_at: 0,
                    },
                    GameEvent::Deal {
                        north: c!(A764S A96H AJD K863C),
                        east: c!(JT953S QT4H K93D ATC),
                        south: c!(2S 875H T542D QJ752C),
                        west: c!(KQ8S KJ32H Q876D 94C),
                        pass: PassDirection::Left,
                    },
                ],
            )?;
            Ok(())
        })?;

        assert!(matches!(
            server.pass_cards(game_id, *TWILSON, c!(A73S)).await,
            Err(CardsError::NotYourCards(c)) if c == c!(3S)
        ));

        assert!(matches!(
            server.pass_cards(game_id, *TWILSON, c!(A7S A9H)).await,
            Err(CardsError::IllegalPassSize(c)) if c == c!(A7S A9H)
        ));

        assert!(matches!(
            server.charge_cards(game_id, *TWILSON, c!(AH)).await,
            Err(CardsError::IllegalAction("charge", phase)) if phase == GamePhase::PassLeft
        ));

        assert!(matches!(
            server.play_card(game_id, *TWILSON, Card::SixClubs).await,
            Err(CardsError::IllegalAction("play", phase)) if phase == GamePhase::PassLeft
        ));

        server.pass_cards(game_id, *TWILSON, c!(K86C)).await?;

        assert!(matches!(
            server.pass_cards(game_id, *TWILSON, c!(A96H)).await,
            Err(CardsError::AlreadyPassed(c)) if c == c!(K86C)
        ));

        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_bot_game() -> Result<(), CardsError> {
    async fn test(_: Database, server: Server) -> Result<(), CardsError> {
        let fake_user = UserId::new();
        let game_id = server.new_game(fake_user, ChargingRules::Classic).await;
        server
            .join_game(
                game_id,
                Player::Bot {
                    user_id: *TWILSON,
                    algorithm: "random".to_string(),
                },
                ChargingRules::BlindChain,
            )
            .await?;
        server.leave_game(game_id, fake_user).await;
        server
            .join_game(
                game_id,
                Player::Bot {
                    user_id: *TSLATCHER,
                    algorithm: "duck".to_string(),
                },
                ChargingRules::Classic,
            )
            .await?;
        server
            .join_game(
                game_id,
                Player::Bot {
                    user_id: *CARRINO,
                    algorithm: "duck".to_string(),
                },
                ChargingRules::Bridge,
            )
            .await?;
        server
            .join_game(
                game_id,
                Player::Bot {
                    user_id: *DCERVELLI,
                    algorithm: "gottatry".to_string(),
                },
                ChargingRules::Blind,
            )
            .await?;
        let mut rx = server.subscribe_game(game_id, UserId::new()).await?;
        let mut plays = 0;
        while let Some(event) = rx.recv().await {
            match event {
                GameEvent::Play { .. } => {
                    plays += 1;
                }
                GameEvent::GameComplete => break,
                _ => {}
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
