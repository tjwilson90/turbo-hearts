use crate::{RedirectToAuthChooser, CONFIG};
use http::{header, Response, StatusCode};
use rusqlite::ErrorCode;
use std::convert::Infallible;
use thiserror::Error;
use turbo_hearts_api::{GameId, RulesError, UserId};
use warp::{reject::Reject, Rejection, Reply};

#[derive(Debug, Error)]
pub enum CardsError {
    #[error("game {0} has already started")]
    GameHasStarted(GameId),
    #[error("Game {0} hasn't completed yet")]
    IncompleteGame(GameId),
    #[error("{0} is not a member of game {1}")]
    InvalidPlayer(UserId, GameId),
    #[error("Games need at least 4 players to start")]
    NotEnoughPlayers,
    #[error("unexpected rules error")]
    Rules {
        #[from]
        source: RulesError,
    },
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
    #[error("{0} is not a known auth token")]
    UnknownAuthToken(String),
    #[error("{0} is not a known game id")]
    UnknownGame(GameId),
}

impl CardsError {
    pub fn is_retriable(&self) -> bool {
        matches!(self, CardsError::Sqlite {
                source: rusqlite::Error::SqliteFailure(e, _),
                ..
            } if e.code == ErrorCode::DatabaseBusy || e.code == ErrorCode::DatabaseLocked)
    }
}

impl Reject for CardsError {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    Ok(if let Some(_) = err.find::<RedirectToAuthChooser>() {
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("{}/auth", &CONFIG.external_uri))
            .body(String::new())
            .unwrap()
    } else if let Some(error) = err.find::<CardsError>() {
        let status = match error {
            CardsError::Serde { .. } | CardsError::Sqlite { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            CardsError::UnknownGame { .. } => StatusCode::NOT_FOUND,
            CardsError::UnknownAuthToken { .. } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::BAD_REQUEST,
        };
        Response::builder()
            .status(status)
            .body(error.to_string())
            .unwrap()
    } else if err.is_not_found() {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(String::new())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(format!("{:?}", err))
            .unwrap()
    })
}
