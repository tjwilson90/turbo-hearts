use crate::{
    cards::{Card, Cards},
    endpoint,
    server::Server,
    types::{GameId, Seat, UserId},
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

pub fn subscribe(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        game_id: GameId,
        server: Server,
        user_id: UserId,
    ) -> Result<impl Reply, Rejection> {
        let rx = server.subscribe_game(game_id, user_id).await?;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("game" / "subscribe" / GameId)
        .and(warp::get())
        .and(server)
        .and(user_id)
        .and_then(handle)
}

pub fn pass_cards(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        cards: Cards,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, cards } = request;
        server.pass_cards(game_id, user_id, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "pass")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn charge_cards(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        cards: Cards,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, cards } = request;
        server.charge_cards(game_id, user_id, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "charge")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn play_card(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        card: Card,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, card } = request;
        server.play_card(game_id, user_id, card).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "play")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id } = request;
        server.claim(game_id, user_id).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn accept_claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        claimer: Seat,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, claimer } = request;
        server.accept_claim(game_id, user_id, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "accept_claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn reject_claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        claimer: Seat,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, claimer } = request;
        server.reject_claim(game_id, user_id, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "reject_claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

pub fn chat(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        message: String,
    }

    async fn handle(
        server: Server,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, message } = request;
        server.game_chat(game_id, user_id, message).await?;
        Ok(warp::reply())
    }

    warp::path!("game" / "chat")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
