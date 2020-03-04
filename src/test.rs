use crate::{
    bot::Strategy,
    card::Card,
    cards::Cards,
    db::Database,
    error::CardsError,
    game::{event::GameEvent, id::GameId, persist_events, phase::GamePhase, Games},
    lobby::{event::LobbyEvent, Lobby},
    player::{Player, PlayerWithOptions},
    seed::Seed,
    types::{ChargingRules, PassDirection},
    user::UserId,
};
use log::LevelFilter;
use once_cell::sync::Lazy;
use r2d2_sqlite::SqliteConnectionManager;
use std::future::Future;
use tempfile::TempDir;

macro_rules! h {
    ($user_id:expr) => {
        crate::player::Player::Human { user_id: $user_id }
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
    lobby: Lobby,
    games: Games,
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
        let lobby = Lobby::new(db.clone()).unwrap();
        let games = Games::new(db.clone(), None);
        Self {
            _temp_dir: temp_dir,
            db,
            lobby,
            games,
        }
    }

    async fn run<F, T>(&self, task: T) -> F::Output
    where
        T: FnOnce(Database, Lobby, Games) -> F + Send + 'static,
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let db = self.db.clone();
        let lobby = self.lobby.clone();
        let games = self.games.clone();
        let result = tokio::spawn(async move { task(db, lobby, games).await }).await;
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
async fn test_lobby() -> Result<(), CardsError> {
    async fn test(_: Database, lobby: Lobby, _games: Games) -> Result<(), CardsError> {
        let mut twilson = lobby.subscribe(*TWILSON).await?;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON],
                chat: vec![],
                games: map![],
            })
        );

        let mut carrino = lobby.subscribe(*CARRINO).await?;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::JoinLobby { user_id: *CARRINO })
        );
        assert_eq!(
            carrino.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON, *CARRINO],
                chat: vec![],
                games: map![],
            })
        );

        drop(carrino);
        lobby.ping().await;
        assert_eq!(twilson.recv().await, Some(LobbyEvent::Ping));
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LeaveLobby { user_id: *CARRINO })
        );

        let tslatcher = PlayerWithOptions {
            player: h!(*TSLATCHER),
            rules: ChargingRules::Bridge,
            seat: None,
        };
        let game_id = lobby.new_game(tslatcher, None).await?;
        match twilson.recv().await {
            Some(LobbyEvent::NewGame {
                game_id: id,
                player,
                seed,
            }) => {
                assert_eq!(id, game_id);
                assert_eq!(player, tslatcher);
                assert_eq!(seed, Seed::Redacted);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        drop(twilson);
        lobby.ping().await;
        let mut twilson = lobby.subscribe(*TWILSON).await?;
        match twilson.recv().await {
            Some(LobbyEvent::LobbyState {
                subscribers,
                chat,
                games,
            }) => {
                assert_eq!(subscribers, set![*TWILSON]);
                assert_eq!(chat, vec![]);
                assert_eq!(games.keys().cloned().collect::<Vec<_>>(), vec![game_id]);
                let game = &games[&game_id];
                assert_eq!(game.players, set![tslatcher]);
                assert_eq!(game.seed, Seed::Redacted);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        let dcervelli = PlayerWithOptions {
            player: h!(*DCERVELLI),
            rules: ChargingRules::Classic,
            seat: None,
        };
        lobby.join_game(game_id, dcervelli).await?;
        match twilson.recv().await {
            Some(LobbyEvent::JoinGame {
                game_id: id,
                player,
            }) => {
                assert_eq!(id, game_id);
                assert_eq!(player, dcervelli);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        lobby.leave_game(game_id, *TSLATCHER).await?;
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
    async fn test(_: Database, lobby: Lobby, games: Games) -> Result<(), CardsError> {
        let game_id = lobby
            .new_game(
                PlayerWithOptions {
                    player: h![*TWILSON],
                    rules: ChargingRules::Classic,
                    seat: None,
                },
                None,
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*TSLATCHER],
                    rules: ChargingRules::Classic,
                    seat: None,
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*DCERVELLI],
                    rules: ChargingRules::Classic,
                    seat: None,
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*CARRINO],
                    rules: ChargingRules::Classic,
                    seat: None,
                },
            )
            .await?;
        games.start_game(game_id)?;
        lobby.start_game(game_id).await;

        let mut twilson = games.subscribe(game_id, *TWILSON).await?;
        match twilson.recv().await {
            Some(GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules: ChargingRules::Classic,
                ..
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
    async fn test(db: Database, _lobby: Lobby, games: Games) -> Result<(), CardsError> {
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
                        created_time: 0,
                        created_by: *TWILSON,
                        seed: Seed::random(),
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
            games.pass_cards(game_id, *TWILSON, c!(A73S)).await,
            Err(CardsError::NotYourCards(c)) if c == c!(3S)
        ));

        assert!(matches!(
            games.pass_cards(game_id, *TWILSON, c!(A7S A9H)).await,
            Err(CardsError::IllegalPassSize(c)) if c == c!(A7S A9H)
        ));

        assert!(matches!(
            games.charge_cards(game_id, *TWILSON, c!(AH)).await,
            Err(CardsError::IllegalAction("charge", phase)) if phase == GamePhase::PassLeft
        ));

        assert!(matches!(
            games.play_card(game_id, *TWILSON, Card::SixClubs).await,
            Err(CardsError::IllegalAction("play", phase)) if phase == GamePhase::PassLeft
        ));

        games.pass_cards(game_id, *TWILSON, c!(K86C)).await?;

        assert!(matches!(
            games.pass_cards(game_id, *TWILSON, c!(A96H)).await,
            Err(CardsError::AlreadyPassed(c)) if c == c!(K86C)
        ));

        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(threaded_scheduler)]
async fn test_bot_game() -> Result<(), CardsError> {
    async fn test(_: Database, lobby: Lobby, games: Games) -> Result<(), CardsError> {
        let game_id = lobby
            .new_game(
                PlayerWithOptions {
                    player: Player::Bot {
                        user_id: *TWILSON,
                        strategy: Strategy::Random,
                    },
                    rules: ChargingRules::BlindChain,
                    seat: None,
                },
                None,
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: Player::Bot {
                        user_id: *TSLATCHER,
                        strategy: Strategy::Duck,
                    },
                    rules: ChargingRules::Classic,
                    seat: None,
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: Player::Bot {
                        user_id: *CARRINO,
                        strategy: Strategy::Duck,
                    },
                    rules: ChargingRules::Bridge,
                    seat: None,
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: Player::Bot {
                        user_id: *DCERVELLI,
                        strategy: Strategy::GottaTry,
                    },
                    rules: ChargingRules::Blind,
                    seat: None,
                },
            )
            .await?;
        games.start_game(game_id)?;
        lobby.start_game(game_id).await;
        let mut rx = games.subscribe(game_id, UserId::new()).await?;
        let mut plays = 0;
        while let Some(event) = rx.recv().await {
            match event {
                GameEvent::Play { .. } => {
                    plays += 1;
                }
                GameEvent::GameComplete { .. } => break,
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
