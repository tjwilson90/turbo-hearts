#![feature(backtrace, drain_filter)]

use crate::{
    cards::{GameId, Player, Seat},
    error::CardsError,
    lobby::Lobby,
    publish::EventKind,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rand::{seq::SliceRandom, RngCore};
use rusqlite::{Connection, Error, ToSql, Transaction, TransactionBehavior, NO_PARAMS};
use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    stream::{Stream, StreamExt},
    sync::{mpsc, mpsc::UnboundedSender, Mutex},
    task, time,
    time::Duration,
};
use warp::{filters::sse::ServerSentEvent, reject, reply, sse, Filter, Rejection, Reply};

mod cards;
mod error;
mod lobby;
mod publish;
mod subscribe;

#[derive(Clone)]
struct Server {
    db: Pool<SqliteConnectionManager>,
    lobby: Lobby,
    subscribers:
        Arc<Mutex<HashMap<GameId, HashMap<Player, UnboundedSender<Option<subscribe::Event>>>>>>,
}

impl Server {
    fn new(db: Pool<SqliteConnectionManager>) -> Self {
        Self {
            db,
            lobby: Lobby::new(),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn run_sql<F, T>(&self, f: F) -> Result<T, Rejection>
    where
        F: FnOnce(Transaction) -> Result<T, rusqlite::Error>,
    {
        task::block_in_place(|| {
            let mut conn = self.db.get().unwrap();
            conn.busy_timeout(Duration::from_secs(5))?;
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            f(tx)
        })
        .map_err(|err: rusqlite::Error| reject::custom(CardsError::from(err)))
    }
}

async fn publish(event: publish::Event, server: Server) -> Result<impl Reply, Infallible> {
    task::block_in_place(move || println!("Published {:?}", event));
    Ok(reply::json(&0))
}

async fn subscribe(
    game_id: GameId,
    player: Player,
    server: Server,
) -> Result<impl Reply, Infallible> {
    let (tx, rx) = mpsc::unbounded_channel();

    // send events so far

    {
        let mut all_subs = server.subscribers.lock().await;
        let subs = all_subs.entry(game_id).or_insert(HashMap::new());
        subs.insert(player, tx);
    }
    Ok(sse::reply(game_stream(rx)))
}

fn game_stream(
    rx: mpsc::UnboundedReceiver<Option<subscribe::Event>>,
) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>> {
    rx.map(|event| {
        if let Some(event) = event {
            let id = event.event_id;
            Ok((sse::json(event), sse::id(id)).into_a())
        } else {
            Ok(sse::comment(String::new()).into_b())
        }
    })
}

async fn enter_lobby(player: Player, server: Server) -> Result<impl Reply, Infallible> {
    let (tx, rx) = mpsc::unbounded_channel();
    server.lobby.enter(player, tx).await;
    Ok(sse::reply(lobby_stream(rx)))
}

fn lobby_stream(
    rx: mpsc::UnboundedReceiver<lobby::Event>,
) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>> {
    rx.map(|event| {
        Ok(if event.is_ping() {
            sse::comment(String::new()).into_a()
        } else {
            sse::json(event).into_b()
        })
    })
}

async fn create_game(player: Player, server: Server) -> Result<impl Reply, Infallible> {
    let id = GameId::new();
    server.lobby.create_game(id, player).await;
    Ok(warp::reply::json(&id))
}

async fn join_game(id: GameId, player: Player, server: Server) -> Result<impl Reply, Rejection> {
    match server.lobby.join_game(id, player).await {
        Ok(mut players) => {
            if players.len() == 4 {
                players.shuffle(&mut rand::thread_rng());
                let timestamp = timestamp();
                server.run_sql(|tx| {
                    let mut stmt = tx.prepare(
                        "INSERT INTO event (game_id, event_id, seat, timestamp, event)
                        VALUES (?, ?, ?, ?, ?)",
                    )?;
                    for i in 0..4 {
                        stmt.execute::<&[&dyn ToSql]>(&[
                            &id,
                            &(i as i64),
                            &Seat::from(i),
                            &timestamp,
                            &subscribe::EventKind::Sit(players[i].clone()),
                        ])?;
                    }
                    Ok(())
                })?;
            }
            Ok(warp::reply::json(&players))
        }
        Err(e) => Err(reject::custom(e)),
    }
}

//async fn todo_create_game(player: Player, server: Server) -> Result<impl Reply, Rejection> {
//    let game_id: GameId = server.run_sql(|tx| {
//        let game_id = tx
//            .query_row("SELECT coalesce(max(id), 0) FROM game", NO_PARAMS, |row| {
//                row.get::<usize, GameId>(0)
//            })?
//            .next();
//        tx.execute::<&[&dyn ToSql]>(
//            "INSERT INTO game (id, timestamp) VALUES (?, ?)",
//            &[&game_id, &timestamp()],
//        )?;
//        Ok(game_id)
//    })?;
//    server.lobby.create_game(game_id, player).await;
//    Ok(warp::reply::json(&game_id))
//}

#[tokio::main]
async fn main() {
    env_logger::init();

    let manager = SqliteConnectionManager::file("cards.db");
    let db = Pool::new(manager).unwrap();
    seed_database(&db.get().unwrap()).unwrap();

    let server = Server::new(db);
    task::spawn(ping_lobby(server.clone()));
    let server = warp::any().map(move || server.clone());

    let publish = warp::path("publish")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(server.clone())
        .and_then(publish);
    let subscribe = warp::path("subscribe")
        .and(warp::path::param::<GameId>())
        .and(warp::path::param::<Player>())
        .and(warp::path::end())
        .and(warp::get())
        .and(server.clone())
        .and_then(subscribe);
    let enter_lobby = warp::path("lobby")
        .and(warp::path::param::<Player>())
        .and(warp::path::end())
        .and(warp::get())
        .and(server.clone())
        .and_then(enter_lobby);
    let create_game = warp::path("create_game")
        .and(warp::path::param::<Player>())
        .and(warp::path::end())
        .and(warp::post())
        .and(server.clone())
        .and_then(create_game);
    let join_game = warp::path("join_game")
        .and(warp::path::param::<GameId>())
        .and(warp::path::param::<Player>())
        .and(warp::path::end())
        .and(warp::post())
        .and(server)
        .and_then(join_game)
        .recover(error::handle_rejection);
    let app = publish
        .or(subscribe)
        .or(enter_lobby)
        .or(create_game)
        .or(join_game);
    warp::serve(app).run(([127, 0, 0, 1], 7380)).await;
}

async fn ping_lobby(server: Server) {
    let mut stream = time::interval(Duration::from_secs(15));
    while let Some(_) = stream.next().await {
        server.lobby.ping().await;
    }
}

fn seed_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS event (
            game_id TEXT NOT NULL,
            event_id INTEGER NOT NULL,
            seat INTEGER NOT NULL,
            timestamp INTEGER NOT NULL,
            event TEXT NOT NULL,
            PRIMARY KEY (game_id, event_id)
        );
        END;",
    )
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
