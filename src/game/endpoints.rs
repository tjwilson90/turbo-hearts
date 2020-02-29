use crate::{
    auth,
    cards::{Card, Cards},
    endpoint,
    server::Server,
    types::{GameId, Seat, UserId},
};
use serde::Deserialize;
use warp::{sse, Filter, Rejection, Reply};

pub fn router(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    warp::path("game")
        .and(
            html()
                .or(subscribe(server.clone(), user_id.clone()))
                .or(pass_cards(server.clone(), user_id.clone()))
                .or(charge_cards(server.clone(), user_id.clone()))
                .or(play_card(server.clone(), user_id.clone()))
                .or(claim(server.clone(), user_id.clone()))
                .or(accept_claim(server.clone(), user_id.clone()))
                .or(reject_claim(server.clone(), user_id.clone()))
                .or(chat(server, user_id)),
        )
        .boxed()
}

fn html() -> reply!() {
    warp::path::end()
        .and(warp::get())
        .and(auth::redirect_if_necessary())
        .untuple_one()
        .and(warp::fs::file("./assets/game/index.html"))
}

fn subscribe(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        game_id: GameId,
        server: Server,
        user_id: UserId,
    ) -> Result<impl Reply, Rejection> {
        let rx = server.subscribe_game(game_id, user_id).await?;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("subscribe" / GameId)
        .and(warp::get())
        .and(server)
        .and(user_id)
        .and_then(handle)
}

fn pass_cards(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("pass")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn charge_cards(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("charge")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn play_card(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("play")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn accept_claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("accept_claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn reject_claim(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("reject_claim")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn chat(server: infallible!(Server), user_id: rejection!(UserId)) -> reply!() {
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

    warp::path!("chat")
        .and(warp::post())
        .and(server)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
