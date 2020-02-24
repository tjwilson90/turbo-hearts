use crate::{
    cards::{Card, Cards},
    endpoint,
    server::Server,
    types::{GameId, Seat},
};
use serde::Deserialize;
use warp::{sse, Filter, Rejection, Reply};

pub fn html() -> reply!() {
    warp::path!("game")
        .and(warp::get())
        .and(crate::auth_flow())
        .untuple_one()
        .and(warp::fs::file("game.html"))
}

pub fn subscribe(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    async fn handle(id: GameId, server: Server, name: String) -> Result<impl Reply, Rejection> {
        let rx = server.subscribe_game(id, name).await?;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("game" / "subscribe" / GameId)
        .and(warp::get())
        .and(server.clone())
        .and(name.clone())
        .and_then(handle)
}

pub fn pass_cards(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        cards: Cards,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, cards } = request;
        server.pass_cards(id, &name, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "pass")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn charge_cards(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        cards: Cards,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, cards } = request;
        server.charge_cards(id, &name, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "charge")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn play_card(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        card: Card,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, card } = request;
        server.play_card(id, &name, card).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "play")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn claim(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id } = request;
        server.claim(id, &name).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "claim")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn accept_claim(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        claimer: Seat,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, claimer } = request;
        server.accept_claim(id, &name, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "accept_claim")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn reject_claim(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        claimer: Seat,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, claimer } = request;
        server.reject_claim(id, &name, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "reject_claim")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn chat(server: infallible!(Server), name: rejection!(String)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        id: GameId,
        message: String,
    }

    async fn handle(
        server: Server,
        name: String,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { id, message } = request;
        server.game_chat(id, name, message).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "chat")
        .and(warp::post())
        .and(server)
        .and(name)
        .and(warp::body::json())
        .and_then(handle)
}
