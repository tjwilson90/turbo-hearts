use crate::{
    auth::AuthFlow, config::CONFIG, db::Database, error::CardsError, server::Server, types::UserId,
    user::Users,
};
use r2d2_sqlite::SqliteConnectionManager;
use rand_distr::Gamma;
use reqwest::Client;
use tokio::{stream::StreamExt, task, time, time::Duration};
use warp::{path::FullPath, Filter, Rejection};

#[macro_use]
mod macros;

mod auth;
mod bot;
mod cards;
mod config;
mod db;
mod endpoint;
mod error;
mod game;
mod lobby;
mod server;
#[cfg(test)]
mod test;
mod types;
mod user;

async fn ping_event_streams(server: Server) {
    let mut stream = time::interval(Duration::from_secs(15));
    while let Some(_) = stream.next().await {
        server.ping_event_streams().await;
    }
}

pub fn auth_flow() -> rejection!(()) {
    async fn handle(path: FullPath, auth_token: Option<String>) -> Result<(), Rejection> {
        match auth_token {
            Some(_) => Ok(()),
            None => Err(AuthFlow(path).into()),
        }
    }

    warp::path::full()
        .and(warp::cookie::optional("AUTH_TOKEN"))
        .and_then(handle)
}

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
    task::spawn(ping_event_streams(server.clone()));

    let server = warp::any().map(move || server.clone());
    let users = warp::any().map(move || users.clone());
    let http_client = warp::any().map(move || http_client.clone());
    let user_id = user_id(users.clone());

    let app = endpoint::lobby::html()
        .or(endpoint::lobby::subscribe(server.clone(), user_id.clone()))
        .or(endpoint::lobby::new_game(server.clone(), user_id.clone()))
        .or(endpoint::lobby::join_game(server.clone(), user_id.clone()))
        .or(endpoint::lobby::leave_game(server.clone(), user_id.clone()))
        .or(endpoint::lobby::add_bot(server.clone()))
        .boxed()
        .or(endpoint::lobby::remove_bot(server.clone()))
        .or(endpoint::lobby::chat(server.clone(), user_id.clone()))
        .or(endpoint::game::html())
        .or(endpoint::game::subscribe(server.clone(), user_id.clone()))
        .or(endpoint::game::pass_cards(server.clone(), user_id.clone()))
        .or(endpoint::game::charge_cards(
            server.clone(),
            user_id.clone(),
        ))
        .boxed()
        .or(endpoint::game::play_card(server.clone(), user_id.clone()))
        .or(endpoint::game::claim(server.clone(), user_id.clone()))
        .or(endpoint::game::accept_claim(
            server.clone(),
            user_id.clone(),
        ))
        .or(endpoint::game::reject_claim(
            server.clone(),
            user_id.clone(),
        ))
        .or(endpoint::game::chat(server.clone(), user_id.clone()))
        .or(endpoint::assets())
        .boxed()
        .or(endpoint::users(users.clone()))
        .or(auth::auth_redirect(users, http_client))
        .recover(error::handle_rejection);
    warp::serve(app).run(([127, 0, 0, 1], CONFIG.port)).await;
    Ok(())
}
