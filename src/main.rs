use crate::{
    config::CONFIG,
    db::Database,
    error::CardsError,
    game::Games,
    lobby::Lobby,
    user::{UserId, Users},
};
use http::header;
use r2d2_sqlite::SqliteConnectionManager;
use rand_distr::Gamma;
use reqwest::Client;
use tokio::{stream::StreamExt, time, time::Duration};
use warp::{Filter, Rejection};

#[macro_use]
mod macros;

mod auth;
mod bot;
mod card;
mod cards;
mod config;
mod db;
mod endpoint;
mod error;
mod game;
mod lobby;
mod player;
mod rank;
mod seat;
mod seed;
mod sql_types;
mod suit;
#[cfg(test)]
mod test;
mod types;
mod user;
mod util;

pub fn user_id(users: infallible!(Users)) -> rejection!(UserId) {
    async fn handle(users: Users, auth_token: String) -> Result<UserId, Rejection> {
        Ok(users.get_user_id(auth_token).await?)
    }

    users.and(warp::cookie("AUTH_TOKEN")).and_then(handle)
}

pub fn start_background_pings(lobby: Lobby, games: Games) {
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
    let db = Database::new(SqliteConnectionManager::file(&CONFIG.db_path))?;
    let bot_delay = Gamma::new(2.0, 1.0).unwrap();
    let lobby = Lobby::new(db.clone())?;
    let games = Games::new(db.clone(), Some(bot_delay));
    let users = Users::new(db);
    let http_client = Client::new();
    start_background_pings(lobby.clone(), games.clone());

    let lobby = warp::any().map(move || lobby.clone());
    let games = warp::any().map(move || games.clone());
    let users = warp::any().map(move || users.clone());
    let http_client = warp::any().map(move || http_client.clone());
    let user_id = user_id(users.clone());

    let app = endpoint::assets()
        .or(game::endpoints::router(
            lobby.clone(),
            games.clone(),
            user_id.clone(),
        ))
        .or(lobby::endpoints::router(lobby, games, user_id))
        .or(auth::endpoints::router(users.clone(), http_client))
        .or(endpoint::users(users))
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
