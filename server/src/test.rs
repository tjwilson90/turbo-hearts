use crate::{persist_events, CardsError, Database, Games, Lobby};
use log::LevelFilter;
use once_cell::sync::Lazy;
use std::{collections::HashMap, future::Future};
use tempfile::TempDir;
use turbo_hearts_api::{
    BotStrategy, Card, ChargingRules, GameEvent, GameId, GamePhase, LobbyEvent, PassDirection,
    Player, PlayerWithOptions, RulesError, Seat, Seed, UserId,
};

macro_rules! h {
    ($user_id:expr) => {
        Player::Human { user_id: $user_id }
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

static TWILSON: Lazy<UserId> = Lazy::new(|| UserId::new());
static TSLATCHER: Lazy<UserId> = Lazy::new(|| UserId::new());
static CARRINO: Lazy<UserId> = Lazy::new(|| UserId::new());
static DCERVELLI: Lazy<UserId> = Lazy::new(|| UserId::new());

struct TestRunner {
    _temp_dir: TempDir,
    db: &'static Database,
    lobby: &'static Lobby,
    games: &'static Games,
}

impl TestRunner {
    fn new() -> Self {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Info)
            .filter_module("turbo_hearts_bot", LevelFilter::Debug)
            .is_test(true)
            .try_init();
        let temp_dir = tempfile::tempdir().unwrap();
        let mut path = temp_dir.path().to_owned();
        path.push("test.db");
        let db = Database::new(path).unwrap();
        let db = &*Box::leak(Box::new(db));
        let lobby = Lobby::new(db).unwrap();
        let lobby = &*Box::leak(Box::new(lobby));
        let games = Games::new(db, false);
        let games = &*Box::leak(Box::new(games));
        Self {
            _temp_dir: temp_dir,
            db,
            lobby,
            games,
        }
    }

    async fn run<F, T>(&self, task: T) -> F::Output
    where
        T: FnOnce(&'static Database, &'static Lobby, &'static Games) -> F + Send + 'static,
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let db = self.db;
        let lobby = self.lobby;
        let games = self.games;
        let result = tokio::spawn(async move { task(db, lobby, games).await }).await;
        match result {
            Ok(v) => v,
            Err(e) => std::panic::resume_unwind(e.into_panic()),
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lobby() -> Result<(), CardsError> {
    async fn test(_: &Database, lobby: &Lobby, _: &Games) -> Result<(), CardsError> {
        let mut twilson = lobby.subscribe(*TWILSON).await?;
        assert_eq!(
            twilson.recv().await,
            Some(LobbyEvent::LobbyState {
                subscribers: set![*TWILSON],
                chat: vec![],
                games: HashMap::new(),
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
                games: HashMap::new(),
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
                player: h!(*TSLATCHER),
            })
        );
        Ok(())
    }

    TestRunner::new().run(test).await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_new_game() -> Result<(), CardsError> {
    async fn test(_: &Database, lobby: &Lobby, games: &Games) -> Result<(), CardsError> {
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
        let (players, seed) = lobby.start_game(game_id).await?;
        games.start_game(game_id, players, seed)?;

        let mut twilson = games.subscribe(game_id, *TWILSON, None).await?;
        match twilson.recv().await {
            Some((
                GameEvent::Sit {
                    north,
                    east,
                    south,
                    west,
                    rules: ChargingRules::Classic,
                    ..
                },
                _,
            )) => assert_eq!(
                set![north, east, south, west],
                set![h!(*TWILSON), h!(*TSLATCHER), h!(*DCERVELLI), h!(*CARRINO)]
            ),
            e => assert!(false, "Expected sit event, found {:?}", e),
        }
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pass() -> Result<(), CardsError> {
    async fn test(db: &Database, _lobby: &Lobby, games: &Games) -> Result<(), CardsError> {
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
            Err(CardsError::Rules { source: RulesError::NotYourCards(c) }) if c == c!(3S)
        ));

        assert!(matches!(
            games.pass_cards(game_id, *TWILSON, c!(A7S A9H)).await,
            Err(CardsError::Rules { source: RulesError::IllegalPassSize(c) }) if c == c!(A7S A9H)
        ));

        assert!(matches!(
            games.charge_cards(game_id, *TWILSON, c!(AH)).await,
            Err(CardsError::Rules { source: RulesError::IllegalAction("charge", phase) }) if phase == GamePhase::PassLeft
        ));

        assert!(matches!(
            games.play_card(game_id, *TWILSON, Card::SixClubs).await,
            Err(CardsError::Rules { source: RulesError::IllegalAction("play", phase) }) if phase == GamePhase::PassLeft
        ));

        games.pass_cards(game_id, *TWILSON, c!(K86C)).await?;

        assert!(matches!(
            games.pass_cards(game_id, *TWILSON, c!(A96H)).await,
            Err(CardsError::Rules { source: RulesError::AlreadyPassed(c) }) if c == c!(K86C)
        ));

        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_seeded_game() -> Result<(), CardsError> {
    async fn test(_db: &Database, lobby: &Lobby, games: &Games) -> Result<(), CardsError> {
        let game_id = lobby
            .new_game(
                PlayerWithOptions {
                    player: h![*TWILSON],
                    rules: ChargingRules::Classic,
                    seat: Some(Seat::North),
                },
                Some("2a3ef864-e49e-440b-9f0a-4125c59716ee".to_string()),
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*TSLATCHER],
                    rules: ChargingRules::Classic,
                    seat: Some(Seat::East),
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*DCERVELLI],
                    rules: ChargingRules::Classic,
                    seat: Some(Seat::South),
                },
            )
            .await?;
        lobby
            .join_game(
                game_id,
                PlayerWithOptions {
                    player: h![*CARRINO],
                    rules: ChargingRules::Classic,
                    seat: Some(Seat::West),
                },
            )
            .await?;
        let (players, seed) = lobby.start_game(game_id).await?;
        games.start_game(game_id, players, seed)?;
        games.pass_cards(game_id, *CARRINO, c!(87H 8C)).await?;
        games.pass_cards(game_id, *DCERVELLI, c!(JH J2C)).await?;
        games.pass_cards(game_id, *TWILSON, c!(Q63H)).await?;
        games.pass_cards(game_id, *TSLATCHER, c!(AD KQC)).await?;
        games.charge_cards(game_id, *TWILSON, c!()).await?;
        games.charge_cards(game_id, *DCERVELLI, c!()).await?;
        games.charge_cards(game_id, *CARRINO, c!()).await?;
        games.charge_cards(game_id, *TSLATCHER, c!(AH)).await?;
        games.charge_cards(game_id, *TWILSON, c!()).await?;
        games.charge_cards(game_id, *DCERVELLI, c!()).await?;
        games.charge_cards(game_id, *CARRINO, c!()).await?;
        games.play_card(game_id, *CARRINO, c!(2C)).await?;
        games.play_card(game_id, *TWILSON, c!(8C)).await?;
        games.play_card(game_id, *TSLATCHER, c!(7C)).await?;
        games.play_card(game_id, *DCERVELLI, c!(QC)).await?;
        games.play_card(game_id, *DCERVELLI, c!(8S)).await?;
        games.play_card(game_id, *CARRINO, c!(3S)).await?;
        games.play_card(game_id, *TWILSON, c!(6S)).await?;
        games.play_card(game_id, *TSLATCHER, c!(2S)).await?;
        games.play_card(game_id, *DCERVELLI, c!(AD)).await?;
        games.play_card(game_id, *CARRINO, c!(7D)).await?;
        games.play_card(game_id, *TWILSON, c!(TD)).await?;
        games.chat(game_id, *CARRINO, "no jack".to_string()).await?;
        games.play_card(game_id, *TSLATCHER, c!(6D)).await?;
        games.chat(game_id, *DCERVELLI, "hi".to_string()).await?;
        games.play_card(game_id, *DCERVELLI, c!(4D)).await?;
        games.play_card(game_id, *CARRINO, c!(QD)).await?;
        games.play_card(game_id, *TWILSON, c!(8D)).await?;
        games.play_card(game_id, *TSLATCHER, c!(AH)).await?;
        games.play_card(game_id, *CARRINO, c!(6C)).await?;
        games.play_card(game_id, *TWILSON, c!(9C)).await?;
        games.play_card(game_id, *TSLATCHER, c!(7S)).await?;
        games.play_card(game_id, *DCERVELLI, c!(3C)).await?;
        games.play_card(game_id, *CARRINO, c!(AC)).await?;
        games.play_card(game_id, *TWILSON, c!(TC)).await?;
        games.play_card(game_id, *TSLATCHER, c!(4S)).await?;
        games.play_card(game_id, *DCERVELLI, c!(KC)).await?;
        games.play_card(game_id, *CARRINO, c!(JC)).await?;
        games.play_card(game_id, *TWILSON, c!(4C)).await?;
        games.play_card(game_id, *TSLATCHER, c!(9H)).await?;
        games.play_card(game_id, *DCERVELLI, c!(QS)).await?;
        games.play_card(game_id, *CARRINO, c!(AS)).await?;
        games.play_card(game_id, *TWILSON, c!(TS)).await?;
        games.play_card(game_id, *TSLATCHER, c!(TH)).await?;
        games.play_card(game_id, *DCERVELLI, c!(KS)).await?;
        games.play_card(game_id, *CARRINO, c!(KD)).await?;
        games.play_card(game_id, *TWILSON, c!(5D)).await?;
        games.play_card(game_id, *TSLATCHER, c!(6H)).await?;
        games.play_card(game_id, *DCERVELLI, c!(9S)).await?;
        games.play_card(game_id, *CARRINO, c!(JH)).await?;
        games.play_card(game_id, *TWILSON, c!(8H)).await?;
        games.play_card(game_id, *TSLATCHER, c!(5H)).await?;
        games.play_card(game_id, *DCERVELLI, c!(KH)).await?;
        games.play_card(game_id, *DCERVELLI, c!(4H)).await?;
        games.play_card(game_id, *CARRINO, c!(9D)).await?;
        games.play_card(game_id, *TWILSON, c!(7H)).await?;
        games.play_card(game_id, *TSLATCHER, c!(3H)).await?;
        games.play_card(game_id, *TWILSON, c!(JD)).await?;
        games.play_card(game_id, *TSLATCHER, c!(QH)).await?;
        games.play_card(game_id, *DCERVELLI, c!(JS)).await?;
        games.play_card(game_id, *CARRINO, c!(2D)).await?;
        games.play_card(game_id, *TWILSON, c!(3D)).await?;
        games.play_card(game_id, *TSLATCHER, c!(2H)).await?;
        games.play_card(game_id, *DCERVELLI, c!(5S)).await?;
        games.play_card(game_id, *CARRINO, c!(5C)).await?;
        Ok(())
    }
    TestRunner::new().run(test).await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_bot_game() -> Result<(), CardsError> {
    async fn test(_: &Database, lobby: &Lobby, games: &Games) -> Result<(), CardsError> {
        let game_id = lobby
            .new_game(
                PlayerWithOptions {
                    player: Player::Bot {
                        user_id: *TWILSON,
                        strategy: BotStrategy::Random,
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
                        strategy: BotStrategy::Heuristic,
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
                        strategy: BotStrategy::Duck,
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
                        strategy: BotStrategy::GottaTry,
                    },
                    rules: ChargingRules::Blind,
                    seat: None,
                },
            )
            .await?;
        let (players, seed) = lobby.start_game(game_id).await?;
        games.start_game(game_id, players, seed)?;
        let mut rx = games.subscribe(game_id, UserId::new(), None).await?;
        let mut events = HashMap::new();
        while let Some((event, _)) = rx.recv().await {
            match event {
                GameEvent::Chat { .. } => {
                    *events.entry("Chat").or_insert(0) += 1;
                }
                GameEvent::Sit { .. } => {
                    *events.entry("Sit").or_insert(0) += 1;
                }
                GameEvent::Deal { .. } => {
                    *events.entry("Deal").or_insert(0) += 1;
                }
                GameEvent::StartPassing { .. } => {
                    *events.entry("StartPassing").or_insert(0) += 1;
                }
                GameEvent::SendPass { .. } => {
                    *events.entry("SendPass").or_insert(0) += 1;
                }
                GameEvent::RecvPass { .. } => {
                    *events.entry("RecvPass").or_insert(0) += 1;
                }
                GameEvent::StartCharging { .. } => {
                    *events.entry("StartCharging").or_insert(0) += 1;
                }
                GameEvent::HandComplete { .. } => {
                    *events.entry("HandComplete").or_insert(0) += 1;
                }
                GameEvent::GameComplete { .. } => {
                    *events.entry("GameComplete").or_insert(0) += 1;
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(events.remove("Chat"), None);
        assert_eq!(events.remove("Sit"), Some(1));
        assert_eq!(events.remove("Deal"), Some(4));
        assert!(matches!(events.remove("StartPassing"), Some(x) if x == 3 || x == 4));
        assert!(matches!(events.remove("SendPass"), Some(x) if x == 12 || x == 16));
        assert!(matches!(events.remove("RecvPass"), Some(x) if x == 12 || x == 16));
        assert!(matches!(events.remove("StartCharging"), Some(x) if x == 4 || x == 5));
        assert_eq!(events.remove("HandComplete"), Some(4));
        assert_eq!(events.remove("GameComplete"), Some(1));
        Ok(())
    }
    let runner = TestRunner::new();
    for _ in 0..30 {
        runner.run(test).await?;
    }
    Ok(())
}
