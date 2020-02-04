use crate::{
    cards::{GameId, Player},
    games,
    games::HandState,
};
use serde::Serialize;
use std::{backtrace::Backtrace, convert::Infallible, fmt, fmt::Display};
use thiserror::Error;
use uuid::Uuid;
use warp::{http::StatusCode, reject::Reject, Rejection};

#[derive(Debug, Error)]
pub enum CardsError {
    #[error("game {0} is already complete")]
    GameComplete(GameId),
    #[error("invalid charging rules: {0}")]
    InvalidChargingRules(String),
    #[error("cannot perform action, currently {0:?}")]
    IllegalAction(HandState),
    #[error("{0} is not a member of the game")]
    IllegalPlayer(Player),
    #[error("unexpected sqlite error")]
    Sqlite {
        #[from]
        source: rusqlite::Error,
        backtrace: Backtrace,
    },
    #[error("unknown game: {0}")]
    UnknownGame(GameId),
}

impl CardsError {
    fn status_code(&self) -> StatusCode {
        match self {
            CardsError::GameComplete(_) => StatusCode::BAD_REQUEST,
            CardsError::IllegalAction(_) => StatusCode::BAD_REQUEST,
            CardsError::IllegalPlayer(_) => StatusCode::BAD_REQUEST,
            CardsError::InvalidChargingRules(_) => StatusCode::BAD_REQUEST,
            CardsError::Sqlite { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CardsError::UnknownGame { .. } => StatusCode::NOT_FOUND,
        }
    }
}

impl Reject for CardsError {}

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub async fn handle_rejection(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "not found".to_string();
    } else if let Some(error) = err.find::<CardsError>() {
        code = error.status_code();
        message = error.to_string();
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "unknown error".to_string();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}
