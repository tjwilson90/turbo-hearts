use crate::{
    games::HandState,
    types::{GameId, Player},
};
use rusqlite::ErrorCode;
use serde::Serialize;
use std::{backtrace::Backtrace, convert::Infallible};
use thiserror::Error;
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
    #[error("unexpected serde error")]
    Serde {
        #[from]
        source: serde_json::Error,
        backtrace: Backtrace,
    },
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
    pub fn is_retriable(&self) -> bool {
        if let CardsError::Sqlite { source, .. } = self {
            if let rusqlite::Error::SqliteFailure(e, _) = source {
                return e.code == ErrorCode::DatabaseBusy || e.code == ErrorCode::DatabaseLocked;
            }
        }
        false
    }

    fn status_code(&self) -> StatusCode {
        match self {
            CardsError::GameComplete(_) => StatusCode::BAD_REQUEST,
            CardsError::IllegalAction(_) => StatusCode::BAD_REQUEST,
            CardsError::IllegalPlayer(_) => StatusCode::BAD_REQUEST,
            CardsError::InvalidChargingRules(_) => StatusCode::BAD_REQUEST,
            CardsError::Serde { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CardsError::Sqlite { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            CardsError::UnknownGame { .. } => StatusCode::NOT_FOUND,
        }
    }
}

impl Reject for CardsError {}

impl From<CardsError> for Rejection {
    fn from(err: CardsError) -> Self {
        warp::reject::custom(err)
    }
}

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
