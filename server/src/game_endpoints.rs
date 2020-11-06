use crate::{auth_redirect, CardsReject, Games, Lobby};
use tokio::{
    stream::{Stream, StreamExt},
    sync::mpsc::UnboundedReceiver,
};
use turbo_hearts_api::{
    AcceptClaimRequest, ChargeRequest, ClaimRequest, GameChatRequest, GameEvent, GameId,
    PassRequest, PlayRequest, RejectClaimRequest, UserId,
};
use warp::{sse, sse::ServerSentEvent, Filter, Rejection, Reply};

pub fn router<'a>(
    lobby: infallible!(&'a Lobby),
    games: infallible!(&'a Games),
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
        .and(auth_redirect())
        .untuple_one()
        .and(warp::fs::file("./assets/game/index.html"))
}

fn subscribe<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        game_id: GameId,
        games: &Games,
        user_id: UserId,
        last_event_id: Option<usize>,
    ) -> Result<impl Reply, Rejection> {
        let rx = games
            .subscribe(game_id, user_id, last_event_id)
            .await
            .map_err(CardsReject)?;
        Ok(sse::reply(stream(rx)))
    }

    fn stream(
        rx: UnboundedReceiver<(GameEvent, usize)>,
    ) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>> {
        rx.map(|(event, id)| {
            if event.is_ping() {
                return Ok(sse::comment(String::new()).into_a());
            }
            if event.is_stable() {
                return Ok((sse::json(event), sse::id(id)).into_a().into_b());
            }
            Ok(sse::json(event).into_b().into_b())
        })
    }

    warp::path!("subscribe" / GameId)
        .and(warp::get())
        .and(games)
        .and(user_id)
        .and(warp::sse::last_event_id())
        .and_then(handle)
}

fn pass_cards<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: PassRequest,
    ) -> Result<impl Reply, Rejection> {
        let PassRequest { game_id, cards } = request;
        games
            .pass_cards(game_id, user_id, cards)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("pass")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn charge_cards<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: ChargeRequest,
    ) -> Result<impl Reply, Rejection> {
        let ChargeRequest { game_id, cards } = request;
        games
            .charge_cards(game_id, user_id, cards)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("charge")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn play_card<'a>(
    lobby: infallible!(&'a Lobby),
    games: infallible!(&'a Games),
    user_id: rejection!(UserId),
) -> reply!() {
    async fn handle(
        lobby: &Lobby,
        games: &Games,
        user_id: UserId,
        request: PlayRequest,
    ) -> Result<impl Reply, Rejection> {
        let PlayRequest { game_id, card } = request;
        let complete = games
            .play_card(game_id, user_id, card)
            .await
            .map_err(CardsReject)?;
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

fn claim<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: ClaimRequest,
    ) -> Result<impl Reply, Rejection> {
        let ClaimRequest { game_id } = request;
        games.claim(game_id, user_id).await.map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn accept_claim<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: AcceptClaimRequest,
    ) -> Result<impl Reply, Rejection> {
        let AcceptClaimRequest { game_id, claimer } = request;
        games
            .accept_claim(game_id, user_id, claimer)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("accept_claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn reject_claim<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: RejectClaimRequest,
    ) -> Result<impl Reply, Rejection> {
        let RejectClaimRequest { game_id, claimer } = request;
        games
            .reject_claim(game_id, user_id, claimer)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("reject_claim")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn chat<'a>(games: infallible!(&'a Games), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(
        games: &Games,
        user_id: UserId,
        request: GameChatRequest,
    ) -> Result<impl Reply, Rejection> {
        let GameChatRequest { game_id, message } = request;
        games
            .chat(game_id, user_id, message)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("chat")
        .and(warp::post())
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}