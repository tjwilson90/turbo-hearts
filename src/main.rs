#![feature(async_closure, backtrace)]

use crate::{
    cards::{Card, Cards},
    db::Database,
    error::CardsError,
    hacks::UnboundedReceiver,
    server::Server,
    types::{ChargingRules, Event, GameId, Player},
};
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio::{
    stream::{Stream, StreamExt},
    task, time,
    time::Duration,
};
use warp::{filters::sse::ServerSentEvent, sse, Filter, Rejection, Reply};

mod cards;
mod db;
mod error;
mod game;
mod hacks;
mod lobby;
mod server;
mod types;

async fn ping_event_streams(server: Server) {
    let mut stream = time::interval(Duration::from_secs(15));
    while let Some(_) = stream.next().await {
        server.ping_event_streams().await;
    }
}

async fn subscribe_lobby(server: Server, player: Player) -> Result<impl Reply, Infallible> {
    let rx = server.subscribe_lobby(player).await;
    Ok(sse::reply(as_stream(rx)))
}

#[derive(Debug, Deserialize)]
struct NewGame {
    rules: ChargingRules,
}

async fn new_game(
    server: Server,
    player: Player,
    request: NewGame,
) -> Result<impl Reply, Infallible> {
    let NewGame { rules } = request;
    let id = server.new_game(player, rules).await;
    Ok(warp::reply::json(&id))
}

#[derive(Debug, Deserialize)]
struct JoinGame {
    id: GameId,
    rules: ChargingRules,
}

async fn join_game(
    server: Server,
    player: Player,
    request: JoinGame,
) -> Result<impl Reply, Rejection> {
    let JoinGame { id, rules } = request;
    let players = server.join_game(id, player, rules).await?;
    Ok(warp::reply::json(&players))
}

#[derive(Debug, Deserialize)]
struct LeaveGame {
    id: GameId,
}

async fn leave_game(
    server: Server,
    player: Player,
    request: LeaveGame,
) -> Result<impl Reply, Infallible> {
    let LeaveGame { id } = request;
    server.leave_game(id, player).await;
    Ok(warp::reply())
}

async fn subscribe_game(
    id: GameId,
    server: Server,
    player: Player,
) -> Result<impl Reply, Rejection> {
    let rx = server.subscribe_game(id, player).await?;
    Ok(sse::reply(as_stream(rx)))
}

#[derive(Debug, Deserialize)]
struct PassCards {
    id: GameId,
    cards: Cards,
}

async fn pass_cards(
    server: Server,
    player: Player,
    request: PassCards,
) -> Result<impl Reply, Rejection> {
    let PassCards { id, cards } = request;
    server.pass_cards(id, player, cards).await?;
    Ok(warp::reply())
}

#[derive(Debug, Deserialize)]
struct ChargeCards {
    id: GameId,
    cards: Cards,
}

async fn charge_cards(
    server: Server,
    player: Player,
    request: ChargeCards,
) -> Result<impl Reply, Rejection> {
    let ChargeCards { id, cards } = request;
    server.charge_cards(id, player, cards).await?;
    Ok(warp::reply())
}

#[derive(Debug, Deserialize)]
struct PlayCard {
    id: GameId,
    card: Card,
}

async fn play_card(
    server: Server,
    player: Player,
    request: PlayCard,
) -> Result<impl Reply, Rejection> {
    let PlayCard { id, card } = request;
    server.play_card(id, player, card).await?;
    Ok(warp::reply())
}

fn as_stream<E>(
    rx: UnboundedReceiver<E>,
) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>>
where
    E: Event + Serialize + Send + Sync + 'static,
{
    rx.map(|event| {
        Ok(if event.is_ping() {
            sse::comment(String::new()).into_a()
        } else {
            sse::json(event).into_b()
        })
    })
}

#[tokio::main]
async fn main() -> Result<(), CardsError> {
    env_logger::init();
    let db = Database::new(SqliteConnectionManager::file("turbo-hearts.db"))?;
    let server = Server::new(db)?;
    task::spawn(ping_event_streams(server.clone()));
    let server = warp::any().map(move || server.clone());
    let player = warp::cookie::optional("player").and_then(async move |player: Option<Player>| {
        player.ok_or(warp::reject::custom(CardsError::MissingPlayerCookie))
    });

    let subscribe_lobby = warp::path!("lobby" / "subscribe")
        .and(warp::get())
        .and(server.clone())
        .and(player.clone())
        .and_then(subscribe_lobby);
    let new_game = warp::path!("lobby" / "new")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(new_game);
    let join_game = warp::path!("lobby" / "join")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(join_game);
    let leave_game = warp::path!("lobby" / "leave")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(leave_game);
    let subscribe_game = warp::path!("game" / "subscribe" / GameId)
        .and(warp::get())
        .and(server.clone())
        .and(player.clone())
        .and_then(subscribe_game);
    let pass_cards = warp::path!("game" / "pass")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(pass_cards);
    let charge_cards = warp::path!("game" / "charge")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(charge_cards);
    let play_card = warp::path!("game" / "play")
        .and(warp::post())
        .and(server.clone())
        .and(player.clone())
        .and(warp::body::json())
        .and_then(play_card);
    let lobby_html = warp::path!("lobby")
        .and(warp::get())
        .and(warp::fs::file("./lobby.html"));
    let game_html = warp::path("game")
        .and(warp::get())
        .and(warp::fs::file("./game.html"));
    let assets = warp::path("assets").and(warp::fs::dir("./assets"));
    let app = subscribe_lobby
        .or(new_game)
        .or(join_game)
        .or(leave_game)
        .or(subscribe_game)
        .or(pass_cards)
        .or(charge_cards)
        .or(play_card)
        .or(lobby_html)
        .or(game_html)
        .or(assets)
        .recover(error::handle_rejection);
    warp::serve(app).run(([127, 0, 0, 1], 7380)).await;
    Ok(())
}
