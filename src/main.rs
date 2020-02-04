#![feature(backtrace, drain_filter)]

use crate::{
    cards::{Card, Cards, ChargingRules, GameId, Player, Seat},
    db::Database,
    error::CardsError,
    games::Games,
    lobby::Lobby,
    publish::EventKind,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rand::{seq::SliceRandom, RngCore};
use rusqlite::{Connection, Error, ToSql, Transaction, TransactionBehavior, NO_PARAMS};
use serde::{Deserialize, Serialize};
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
mod db;
mod error;
mod games;
mod lobby;
mod publish;

#[derive(Clone)]
struct Server {
    db: Database,
    lobby: Lobby,
    games: Games,
}

impl Server {
    fn new() -> Self {
        Self {
            db: Database::new(),
            lobby: Lobby::new(),
            games: Games::new(),
        }
    }
}

async fn subscribe_lobby(player: Player, server: Server) -> Result<impl Reply, Infallible> {
    let (tx, rx) = mpsc::unbounded_channel();
    server.lobby.subscribe(player, tx).await;
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

#[derive(Debug, Serialize, Deserialize)]
struct NewGame {
    player: Player,
    rules: ChargingRules,
}

async fn new_game(server: Server, request: NewGame) -> Result<impl Reply, Infallible> {
    let id = GameId::new();
    let NewGame { player, rules } = request;
    server.lobby.new_game(id, player, rules).await;
    Ok(warp::reply::json(&id))
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinGame {
    id: GameId,
    player: Player,
    rules: ChargingRules,
}

async fn join_game(server: Server, request: JoinGame) -> Result<impl Reply, Rejection> {
    let JoinGame { id, player, rules } = request;
    match server.lobby.join_game(id, player, rules).await {
        Ok(mut players) => {
            if players.len() == 4 {
                let mut order = players.keys().cloned().collect::<Vec<_>>();
                order.shuffle(&mut rand::thread_rng());
                server.db.run_with_retry(|tx| {
                    tx.execute::<&[&dyn ToSql]>(
                        "INSERT INTO game (id, timestamp, north, east, south, west, rules)
                        VALUES (?, ?, ?, ?, ?, ?)",
                        &[
                            &id,
                            &timestamp(),
                            &order[0],
                            &order[1],
                            &order[2],
                            &order[3],
                            &players.get(&order[0]).unwrap(),
                        ],
                    )
                })?;
            }
            Ok(warp::reply::json(&players))
        }
        Err(e) => Err(reject::custom(e)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LeaveGame {
    id: GameId,
    player: Player,
}

async fn leave_game(server: Server, request: LeaveGame) -> Result<impl Reply, Infallible> {
    let LeaveGame { id, player } = request;
    server.lobby.leave_game(id, player).await;
    Ok(warp::reply())
}

async fn subscribe_game(
    id: GameId,
    player: Player,
    server: Server,
) -> Result<impl Reply, Infallible> {
    let (tx, rx) = mpsc::unbounded_channel();
    server.games.subscribe(id, player, tx).await;
    Ok(sse::reply(game_stream(rx)))
}

fn game_stream(
    rx: mpsc::UnboundedReceiver<games::Event>,
) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>> {
    rx.map(|event| {
        Ok(if event.is_ping() {
            sse::comment(String::new()).into_a()
        } else {
            sse::json(event).into_b()
        })
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct PassCards {
    id: GameId,
    player: Player,
    cards: Cards,
}

async fn pass_cards(server: Server, request: PassCards) -> Result<impl Reply, Infallible> {
    let PassCards { id, player, cards } = request;
    server.games.pass_cards(id, player, cards).await;
    Ok(warp::reply())
}

#[derive(Debug, Serialize, Deserialize)]
struct ChargeCards {
    id: GameId,
    player: Player,
    cards: Cards,
}

async fn charge_cards(server: Server, request: ChargeCards) -> Result<impl Reply, Infallible> {
    let ChargeCards { id, player, cards } = request;
    server.games.charge_cards(id, player, cards).await;
    Ok(warp::reply())
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayCard {
    id: GameId,
    player: Player,
    card: Card,
}

async fn play_card(server: Server, request: PlayCard) -> Result<impl Reply, Infallible> {
    let PlayCard { id, player, card } = request;
    server.games.play_card(id, player, card).await;
    Ok(warp::reply())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let server = Server::new();
    task::spawn(ping_event_streams(server.clone()));
    let server = warp::any().map(move || server.clone());

    let subscribe_lobby = warp::path!("lobby" / Player)
        .and(warp::get())
        .and(server.clone())
        .and_then(subscribe_lobby);
    let new_game = warp::path!("lobby" / "new")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(new_game);
    let join_game = warp::path!("lobby" / "join")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(join_game);
    let leave_game = warp::path!("lobby" / "leave")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(leave_game);
    let subscribe_game = warp::path!("game" / GameId / Player)
        .and(warp::get())
        .and(server.clone())
        .and_then(subscribe_game);
    let pass_cards = warp::path!("game" / "pass")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(pass_cards);
    let charge_cards = warp::path!("game" / "charge")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(charge_cards);
    let play_card = warp::path!("game" / "play")
        .and(warp::post())
        .and(server.clone())
        .and(warp::body::json())
        .and_then(play_card);
    let app = subscribe_lobby
        .or(new_game)
        .or(join_game)
        .or(leave_game)
        .or(subscribe_game)
        .or(pass_cards)
        .or(charge_cards)
        .or(play_card)
        .recover(error::handle_rejection);
    warp::serve(app).run(([127, 0, 0, 1], 7380)).await;
}

async fn ping_event_streams(server: Server) {
    let mut stream = time::interval(Duration::from_secs(15));
    while let Some(_) = stream.next().await {
        server.lobby.ping().await;
    }
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
