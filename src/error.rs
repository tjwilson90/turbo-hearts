use crate::types::Name;
use crate::{cards::Cards, game::GameState, types::GameId};
use rusqlite::ErrorCode;
use serde::Serialize;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, reject::Reject, Rejection};

#[derive(Debug, Error)]
pub enum CardsError {
    #[error("{0} has already been charged")]
    AlreadyCharged(Cards),
    #[error("{0} has already been passed")]
    AlreadyPassed(Cards),
    #[error("game {0} is already complete")]
    GameComplete(GameId),
    #[error("game {0} has already started")]
    GameHasStarted(GameId),
    #[error("hearts cannot be lead if hearts are not broken")]
    HeartsNotBroken,
    #[error("cannot perform action, currently {0:?}")]
    IllegalAction(GameState),
    #[error("{0} is not a valid name for a human player")]
    IllegalName(Name),
    #[error("{0} is not a legal pass, passes must have 3 cards")]
    IllegalPassSize(Cards),
    #[error("{0} is not a member of the game")]
    IllegalPlayer(Name),
    #[error("charged cards cannot be played on the first trick of their suit")]
    NoChargeOnFirstTrickOfSuit,
    #[error("points cannot be played on the first trick")]
    NoPointsOnFirstTrick,
    #[error("your hand does not contain {0}")]
    NotYourCards(Cards),
    #[error("player {0} makes the next charge or play")]
    NotYourTurn(Name),
    #[error("api endpoints require a \"player\" cookie identifying the caller")]
    MissingPlayerCookie,
    #[error("the first lead must be the two of clubs")]
    MustStartWithTwoOfClubs,
    #[error("suit must be followed")]
    MustFollowSuit,
    #[error("unexpected serde error")]
    Serde {
        #[from]
        source: serde_json::Error,
    },
    #[error("unexpected sqlite error")]
    Sqlite {
        #[from]
        source: rusqlite::Error,
    },
    #[error("the cards {0} cannot be charged")]
    Unchargeable(Cards),
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
            CardsError::Serde { .. } | CardsError::Sqlite { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CardsError::UnknownGame { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
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
        message = format!("{:?}", err);
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}
