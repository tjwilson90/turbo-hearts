use crate::{
    endpoint,
    server::Server,
    types::{ChargingRules, GameId, Player, UserId},
};
use serde::Deserialize;
use std::convert::Infallible;
use warp::{sse, Filter, Rejection, Reply};

pub fn html() -> reply!() {
    warp::path!("lobby")
        .and(warp::get())
        .and(crate::auth_flow())
        .untuple_one()
        .and(warp::fs::file("lobby.html"))
}

pub fn subscribe(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(server: Server, user_id: UserId) -> Result<impl Reply, Infallible> {
        let rx = server.subscribe_lobby(user_id).await;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("lobby" / "subscribe")
        .and(warp::get())
        .and(server)
        .and(user_id)
        .and_then(handle)
}

pub fn new_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        rules: ChargingRules,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Infallible> {
        let Request { rules } = request;
        let game_id = server.new_game(user_id, rules).await;
        Ok(warp::reply::json(&game_id))
    }

    warp::path!("lobby" / "new")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn join_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, rules } = request;
        let players = server
            .join_game(game_id, Player::Human { user_id }, rules)
            .await?;
        Ok(warp::reply::json(&players))
    }

    warp::path!("lobby" / "join")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn leave_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Infallible> {
        let Request { game_id } = request;
        server.leave_game(game_id, user_id).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "leave")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn add_bot(server: infallible!(Server)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
        algorithm: String,
    }

    async fn handle(server: Server, request: Request) -> Result<impl Reply, Rejection> {
        let Request {
            game_id,
            rules,
            algorithm,
        } = request;
        let bot_id = UserId::new();
        let player = Player::Bot {
            user_id: bot_id,
            algorithm,
        };
        let _ = server.join_game(game_id, player, rules).await?;
        Ok(warp::reply::json(&bot_id))
    }

    warp::path!("lobby" / "add_bot")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn remove_bot(server: infallible!(Server)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        user_id: UserId,
    }

    async fn handle(server: Server, request: Request) -> Result<impl Reply, Infallible> {
        let Request { game_id, user_id } = request;
        server.leave_game(game_id, user_id).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "remove_bot")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn chat(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        message: String,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { message } = request;
        server.lobby_chat(user_id, message).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "chat")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
