use crate::{
    config::CONFIG, db::Database, error::CardsReject, game::Games, lobby::Lobby, user::Users,
};
use http::header;
use log::error;
use reqwest::Client;
use tokio::{stream::StreamExt, time, time::Duration};
use turbo_hearts_api::{CardsError, UserId};
use warp::{Filter, Rejection};

#[macro_use]
mod macros;

mod assets;
mod auth;
mod bot;
mod config;
mod db;
mod error;
mod game;
mod lobby;
mod sender;
mod summary;
#[cfg(test)]
mod test;
mod user;
mod util;

pub use sender::*;

fn user_id<'a>(users: infallible!(&'a Users)) -> rejection!(UserId) {
    async fn handle(users: &Users, auth_token: String) -> Result<UserId, Rejection> {
        Ok(users.get_user_id(auth_token).await.map_err(CardsReject)?)
    }

    users.and(warp::cookie("AUTH_TOKEN")).and_then(handle)
}

fn start_stale_game_cleanup(lobby: &'static Lobby) {
    tokio::task::spawn(async move {
        let mut stream = time::interval(Duration::from_secs(60 * 60));
        while let Some(_) = stream.next().await {
            if let Err(e) = lobby.delete_stale_games().await {
                error!("Failed to delete stale games {}", e);
            }
        }
    });
}

fn start_background_pings(lobby: &'static Lobby, games: &'static Games) {
    tokio::task::spawn(async move {
        let mut stream = time::interval(Duration::from_secs(15));
        while let Some(_) = stream.next().await {
            lobby.ping().await;
            games.ping().await;
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), CardsError> {
    env_logger::init();

    let db = Database::new(&CONFIG.db_path)?;
    let db = &*Box::leak(Box::new(db));

    let lobby = Lobby::new(db)?;
    let lobby = &*Box::leak(Box::new(lobby));

    let games = Games::new(db, true);
    let games = &*Box::leak(Box::new(games));

    let users = Users::new(db);
    let users = &*Box::leak(Box::new(users));

    let http_client = Client::new();
    let http_client = &*Box::leak(Box::new(http_client));

    start_stale_game_cleanup(lobby);
    start_background_pings(lobby, games);

    let db = warp::any().map(move || db);
    let lobby = warp::any().map(move || lobby);
    let games = warp::any().map(move || games);
    let users = warp::any().map(move || users);
    let http_client = warp::any().map(move || http_client);
    let user_id = user_id(users);

    let app = assets::router()
        .or(game::endpoints::router(lobby, games, user_id.clone()))
        .or(lobby::endpoints::router(lobby, games, user_id))
        .or(auth::endpoints::router(users, http_client))
        .or(user::endpoints::router(users))
        .or(summary::router(db))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_credentials(true)
                .allow_header(header::CONTENT_TYPE)
                .build(),
        )
        .recover(error::handle_rejection)
        .with(warp::log("request"));
    warp::serve(app).run(([127, 0, 0, 1], CONFIG.port)).await;
    Ok(())
}
