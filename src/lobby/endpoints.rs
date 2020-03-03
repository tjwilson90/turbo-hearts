use crate::{
    auth,
    bot::Strategy,
    endpoint,
    server::Server,
    types::{ChargingRules, GameId, Player, PlayerWithOptions, Seat, UserId},
};
use serde::Deserialize;
use std::convert::Infallible;
use warp::{sse, Filter, Rejection, Reply};

pub fn router(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    warp::path("lobby")
        .and(
            html()
                .or(subscribe(server.clone(), user_id.clone()))
                .or(new_game(server.clone(), user_id.clone()))
                .or(join_game(server.clone(), user_id.clone()))
                .or(leave_game(server.clone(), user_id.clone()))
                .or(add_bot(server.clone()))
                .or(remove(server.clone()))
                .or(chat(server, user_id)),
        )
        .boxed()
}

fn html() -> reply!() {
    warp::path::end()
        .and(warp::get())
        .and(auth::redirect_if_necessary())
        .untuple_one()
        .and(warp::fs::file("./assets/lobby/index.html"))
}

fn subscribe(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(server: Server, user_id: UserId) -> Result<impl Reply, Infallible> {
        let rx = server.subscribe_lobby(user_id).await;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("subscribe")
        .and(warp::get())
        .and(server)
        .and(user_id)
        .and_then(handle)
}

fn new_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        rules: ChargingRules,
        seat: Option<Seat>,
        seed: Option<String>,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Infallible> {
        let Request { rules, seat, seed } = request;
        let player = PlayerWithOptions {
            player: Player::Human { user_id },
            rules,
            seat,
        };
        let game_id = server.new_game(player, seed).await;
        Ok(warp::reply::json(&game_id))
    }

    warp::path!("new")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn join_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
        seat: Option<Seat>,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request {
            game_id,
            rules,
            seat,
        } = request;
        let player = PlayerWithOptions {
            player: Player::Human { user_id },
            rules,
            seat,
        };
        server.join_game(game_id, player).await?;
        Ok(warp::reply())
    }

    warp::path!("join")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn leave_game(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("leave")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn add_bot(server: infallible!(Server)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
        strategy: Strategy,
    }

    async fn handle(server: Server, request: Request) -> Result<impl Reply, Rejection> {
        let Request {
            game_id,
            rules,
            strategy,
        } = request;
        let bot_id = UserId::new();
        let player = PlayerWithOptions {
            player: Player::Bot {
                user_id: bot_id,
                strategy,
            },
            rules,
            seat: None,
        };
        server.join_game(game_id, player).await?;
        Ok(warp::reply::json(&bot_id))
    }

    warp::path!("add_bot")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

fn remove(server: infallible!(Server)) -> reply!() {
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

    warp::path!("remove")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

fn chat(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("chat")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
