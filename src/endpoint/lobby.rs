use crate::{
    bot, endpoint,
    server::Server,
    types::{ChargingRules, GameId, Player},
};
use serde::Deserialize;
use std::convert::Infallible;
use warp::{sse, Filter, Rejection, Reply};

pub fn html() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    warp::path!("lobby")
        .and(warp::get())
        .and(crate::auth_flow())
        .untuple_one()
        .and(warp::fs::file("lobby.html"))
}

pub fn subscribe(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
    name: impl Filter<Extract = (String,), Error = Rejection> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    async fn handle(server: Server, name: String) -> Result<impl Reply, Infallible> {
        let rx = server.subscribe_lobby(name).await;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("lobby" / "subscribe")
        .and(warp::get())
        .and(server)
        .and(name)
        .and_then(handle)
}

pub fn new_game(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
    name: impl Filter<Extract = (String,), Error = Rejection> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        rules: ChargingRules,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Infallible> {
        let Request { rules } = request;
        let id = server.new_game(&name, rules).await;
        Ok(warp::reply::json(&id))
    }

    warp::path!("lobby" / "new")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn join_game(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
    name: impl Filter<Extract = (String,), Error = Rejection> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        rules: ChargingRules,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, rules } = request;
        let players = server.join_game(id, Player::Human { name }, rules).await?;
        Ok(warp::reply::json(&players))
    }

    warp::path!("lobby" / "join")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn leave_game(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
    name: impl Filter<Extract = (String,), Error = Rejection> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Infallible> {
        let Request { id } = request;
        server.leave_game(id, name).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "leave")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn add_bot(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        rules: ChargingRules,
        algorithm: String,
    }

    async fn handle(server: Server, request: Request) -> Result<impl Reply, Rejection> {
        let Request {
            id,
            rules,
            algorithm,
        } = request;
        let name = bot::name();
        let player = Player::Bot {
            name: name.clone(),
            algorithm,
        };
        let _ = server.join_game(id, player, rules).await?;
        Ok(warp::reply::json(&name))
    }

    warp::path!("lobby" / "add_bot")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn remove_bot(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        name: String,
    }

    async fn handle(server: Server, request: Request) -> Result<impl Reply, Infallible> {
        let Request { id, name } = request;
        server.leave_game(id, name).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "remove_bot")
        .and(warp::post())
        .and(server)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn chat(
    server: impl Filter<Extract = (Server,), Error = Infallible> + Clone + Send,
    name: impl Filter<Extract = (String,), Error = Rejection> + Clone + Send,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone + Send {
    #[derive(Debug, Deserialize)]
    struct Request {
        message: String,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { message } = request;
        server.lobby_chat(name, message).await;
        Ok(warp::reply())
    }

    warp::path!("lobby" / "chat")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}
