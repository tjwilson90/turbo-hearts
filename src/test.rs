use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    game::{persist_events, GameDbEvent, GameFeEvent, GameState},
    lobby::LobbyEvent,
    server::Server,
    types::{ChargingRules, GameId},
};
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
    let result = tokio::spawn(async move {
        let dir = tempfile::tempdir().unwrap();
        let mut path = dir.path().to_owned();
        path.push("test.db");
        let db = Database::new(SqliteConnectionManager::file(path)).unwrap();
        task(db).await
    })
    .await;
    match result {
        Ok(v) => v,
        Err(e) => std::panic::resume_unwind(e.into_panic()),
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
    assert_eq!(Cards::from(Card::AceHearts), "AH".parse().unwrap())
}

#[tokio::test(threaded_scheduler)]
async fn test_join_unknown_game() -> Result<(), CardsError> {
    run(async move |db| {
        let server = Server::new(db)?;
        let id = GameId::new();
        let resp = server
            .join_game(id, p!(twilson), ChargingRules::Classic)
            .await;
        assert!(matches!(resp, Err(CardsError::UnknownGame(game)) if game == id));
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
async fn test_new_game() -> Result<(), CardsError> {
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
            Some(GameFeEvent::Sit {
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

#[tokio::test(threaded_scheduler)]
async fn test_pass() -> Result<(), CardsError> {
    run(async move |db| {
        let id = GameId::new();
        db.run_with_retry(|tx| {
            persist_events(
                &tx,
                id,
                0,
                &[
                    GameDbEvent::Sit {
                        north: p!(twilson),
                        east: p!(dcervelli),
                        south: p!(tslatcher),
                        west: p!(carrino),
                        rules: ChargingRules::Classic,
                    },
                    GameDbEvent::Deal {
                        north: "A764S A96H AJD K863C".parse().unwrap(),
                        east: "JT953S QT4H K93D ATC".parse().unwrap(),
                        south: "2S 875H T542D QJ752C".parse().unwrap(),
                        west: "KQ8S KJ32H Q876D 94C".parse().unwrap(),
                    },
                ],
            )?;
            Ok(())
        })?;
        let server = Server::new(db)?;

        let bad_pass = server
            .pass_cards(id, p!(twilson), "A73S".parse().unwrap())
            .await;
        assert!(matches!(
            bad_pass,
            Err(CardsError::NotYourCards(c)) if c == "3S".parse().unwrap()
        ));

        let bad_pass = server
            .pass_cards(id, p!(twilson), "A7S A9H".parse().unwrap())
            .await;
        assert!(matches!(
            bad_pass,
            Err(CardsError::IllegalPassSize(c)) if c == "A7S A9H".parse().unwrap()
        ));

        let bad_charge = server
            .charge_cards(id, p!(twilson), "AH".parse().unwrap())
            .await;
        assert!(matches!(
            bad_charge,
            Err(CardsError::IllegalAction(state)) if state == GameState::Passing
        ));

        let bad_play = server.play_card(id, p!(twilson), Card::SixClubs).await;
        assert!(matches!(
            bad_play,
            Err(CardsError::IllegalAction(state)) if state == GameState::Passing
        ));

        server
            .pass_cards(id, p!(twilson), "K86C".parse().unwrap())
            .await?;

        let bad_pass = server
            .pass_cards(id, p!(twilson), "A96H".parse().unwrap())
            .await;
        assert!(matches!(
            bad_pass,
            Err(CardsError::AlreadyPassed(c)) if c == "K86C".parse().unwrap()
        ));

        Ok(())
    })
    .await
}
