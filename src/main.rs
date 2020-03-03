use crate::{
    config::CONFIG,
    db::Database,
    error::CardsError,
    server::Server,
    user::{UserId, Users},
};
use http::header;
use r2d2_sqlite::SqliteConnectionManager;
use rand_distr::Gamma;
use reqwest::Client;
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
mod rank;
mod seat;
mod server;
mod suit;
#[cfg(test)]
mod test;
mod types;
mod user;

pub fn user_id(users: infallible!(Users)) -> rejection!(UserId) {
    async fn handle(users: Users, auth_token: String) -> Result<UserId, Rejection> {
        Ok(users.get_user_id(auth_token).await?)
    }

    users.and(warp::cookie("AUTH_TOKEN")).and_then(handle)
}

#[tokio::main]
async fn main() -> Result<(), CardsError> {
    env_logger::init();
    let db = Database::new(SqliteConnectionManager::file(&CONFIG.db_path))?;
    let server = Server::with_slow_bots(db.clone(), Gamma::new(2.0, 1.0).unwrap())?;
    let users = Users::new(db);
    let http_client = Client::new();
    server.clone().start_background_pings();

    let server = warp::any().map(move || server.clone());
    let users = warp::any().map(move || users.clone());
    let http_client = warp::any().map(move || http_client.clone());
    let user_id = user_id(users.clone());

    let app = endpoint::assets()
        .or(game::endpoints::router(server.clone(), user_id.clone()))
        .or(lobby::endpoints::router(server, user_id))
        .or(auth::endpoints::router(users.clone(), http_client))
        .or(endpoint::users(users))
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_credentials(true)
                .allow_header(header::CONTENT_TYPE)
                .build(),
        )
        .recover(error::handle_rejection);
    warp::serve(app).run(([127, 0, 0, 1], CONFIG.port)).await;
    Ok(())
}
