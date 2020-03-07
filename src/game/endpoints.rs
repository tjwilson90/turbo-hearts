use crate::{
    auth,
    card::Card,
    cards::Cards,
    endpoint,
    game::{id::GameId, Games},
    lobby::Lobby,
    seat::Seat,
    user::UserId,
};
use serde::Deserialize;
use warp::{sse, Filter, Rejection, Reply};

pub fn router(
    lobby: infallible!(Lobby),
    games: infallible!(Games),
    user_id: rejection!(UserId),
) -> reply!() {
    warp::path("game")
        .and(
            html()
                .or(subscribe(games.clone(), user_id.clone()))
                .or(pass_cards(games.clone(), user_id.clone()))
                .or(charge_cards(games.clone(), user_id.clone()))
                .or(play_card(lobby, games.clone(), user_id.clone()))
                .or(claim(games.clone(), user_id.clone()))
                .or(accept_claim(games.clone(), user_id.clone()))
                .or(reject_claim(games.clone(), user_id.clone()))
                .or(chat(games, user_id)),
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

fn subscribe(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        game_id: GameId,
        games: Games,
        user_id: UserId,
    ) -> Result<impl Reply, Rejection> {
        let rx = games.subscribe(game_id, user_id).await?;
        Ok(sse::reply(endpoint::as_stream(rx)))
    }

    warp::path!("subscribe" / GameId)
        .and(warp::get())
        .and(games)
        .and(user_id)
        .and_then(handle)
}

fn pass_cards(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        cards: Cards,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, cards } = request;
        games.pass_cards(game_id, user_id, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("pass")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn charge_cards(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        cards: Cards,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, cards } = request;
        games.charge_cards(game_id, user_id, cards).await?;
        Ok(warp::reply())
    }

    warp::path!("charge")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn play_card(
    lobby: infallible!(Lobby),
    games: infallible!(Games),
    user_id: rejection!(UserId),
) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        card: Card,
    }

    async fn handle(
        lobby: Lobby,
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, card } = request;
        let complete = games.play_card(game_id, user_id, card).await?;
        if complete {
            lobby.finish_game(game_id).await;
        }
        Ok(warp::reply())
    }

    warp::path!("play")
        .and(warp::post())
        .and(lobby)
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn claim(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id } = request;
        games.claim(game_id, user_id).await?;
        Ok(warp::reply())
    }

    warp::path!("claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn accept_claim(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        claimer: Seat,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, claimer } = request;
        games.accept_claim(game_id, user_id, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("accept_claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn reject_claim(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        claimer: Seat,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, claimer } = request;
        games.reject_claim(game_id, user_id, claimer).await?;
        Ok(warp::reply())
    }

    warp::path!("reject_claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn chat(games: infallible!(Games), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        message: String,
    }

    async fn handle(
        games: Games,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, message } = request;
        games.chat(game_id, user_id, message).await?;
        Ok(warp::reply())
    }

    warp::path!("chat")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
