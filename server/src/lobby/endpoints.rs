use crate::{
    auth,
    game::Games,
    lobby::{event::LobbyEvent, Lobby},
    CardsReject,
};
use serde::Deserialize;
use tokio::{
    stream::{Stream, StreamExt},
    sync::mpsc::UnboundedReceiver,
};
use turbo_hearts_api::{
    BotStrategy, ChargingRules, GameId, Player, PlayerWithOptions, Seat, UserId,
};
use warp::{sse, sse::ServerSentEvent, Filter, Rejection, Reply};

pub fn router<'a>(
    lobby: infallible!(&'a Lobby),
    games: infallible!(&'a Games),
    user_id: rejection!(UserId),
) -> reply!() {
    warp::path("lobby")
        .and(
            html()
                .or(subscribe(lobby.clone(), user_id.clone()))
                .or(new_game(lobby.clone(), user_id.clone()))
                .or(join_game(lobby.clone(), user_id.clone()))
                .or(start_game(lobby.clone(), games, user_id.clone()))
                .or(leave_game(lobby.clone(), user_id.clone()))
                .or(add_bot(lobby.clone(), user_id.clone()))
                .or(remove(lobby.clone(), user_id.clone()))
                .or(chat(lobby, user_id)),
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

fn subscribe<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    async fn handle(lobby: &Lobby, user_id: UserId) -> Result<impl Reply, Rejection> {
        let rx = lobby.subscribe(user_id).await.map_err(CardsReject)?;
        Ok(sse::reply(stream(rx)))
    }

    fn stream(
        rx: UnboundedReceiver<LobbyEvent>,
    ) -> impl Stream<Item = Result<impl ServerSentEvent, warp::Error>> {
        rx.map(|event| {
            Ok(if event.is_ping() {
                sse::comment(String::new()).into_a()
            } else {
                sse::json(event).into_b()
            })
        })
    }

    warp::path!("subscribe")
        .and(warp::get())
        .and(lobby)
        .and(user_id)
        .and_then(handle)
}

fn new_game<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        rules: ChargingRules,
        seat: Option<Seat>,
        seed: Option<String>,
    }

    async fn handle(
        lobby: &Lobby,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { rules, seat, seed } = request;
        let player = PlayerWithOptions {
            player: Player::Human { user_id },
            rules,
            seat,
        };
        let game_id = lobby.new_game(player, seed).await.map_err(CardsReject)?;
        Ok(warp::reply::json(&game_id))
    }

    warp::path!("new")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn join_game<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
        seat: Option<Seat>,
    }

    async fn handle(
        lobby: &Lobby,
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
        lobby
            .join_game(game_id, player)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("join")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn start_game<'a>(
    lobby: infallible!(&'a Lobby),
    games: infallible!(&'a Games),
    user_id: rejection!(UserId),
) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
    }

    async fn handle(
        lobby: &Lobby,
        games: &Games,
        _user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id } = request;
        let (players, seed) = lobby.start_game(game_id).await.map_err(CardsReject)?;
        games
            .start_game(game_id, players, seed)
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("start")
        .and(warp::post())
        .and(lobby)
        .and(games)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn leave_game<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
    }

    async fn handle(
        lobby: &Lobby,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id } = request;
        lobby
            .leave_game(game_id, user_id)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("leave")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn add_bot<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        rules: ChargingRules,
        strategy: BotStrategy,
    }

    async fn handle(
        lobby: &Lobby,
        _user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
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
        lobby
            .join_game(game_id, player)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply::json(&bot_id))
    }

    warp::path!("add_bot")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn remove<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        game_id: GameId,
        user_id: UserId,
    }

    async fn handle(
        lobby: &Lobby,
        _user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { game_id, user_id } = request;
        lobby
            .leave_game(game_id, user_id)
            .await
            .map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("remove")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}

fn chat<'a>(lobby: infallible!(&'a Lobby), user_id: rejection!(UserId)) -> reply!() {
    #[derive(Debug, Deserialize)]
    struct Request {
        message: String,
    }

    async fn handle(
        lobby: &Lobby,
        user_id: UserId,
        request: Request,
    ) -> Result<impl Reply, Rejection> {
        let Request { message } = request;
        lobby.chat(user_id, message).await.map_err(CardsReject)?;
        Ok(warp::reply())
    }

    warp::path!("chat")
        .and(warp::post())
        .and(lobby)
        .and(user_id)
        .and(warp::body::json())
        .and_then(handle)
}
