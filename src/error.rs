use crate::{
    auth::AuthFlow,
    cards::{Cards, GamePhase},
    types::GameId,
};
use http::{Response, StatusCode};
use rusqlite::ErrorCode;
use std::convert::Infallible;
use thiserror::Error;
use warp::{reject::Reject, Rejection, Reply};

#[derive(Debug, Error)]
pub enum CardsError {
    #[error("{0} has already accepted the claim from {1}")]
    AlreadyAcceptedClaim(String, String),
    #[error("{0} has already been charged")]
    AlreadyCharged(Cards),
    #[error("{0} has already made a claim")]
    AlreadyClaiming(String),
    #[error("{0} has already been passed")]
    AlreadyPassed(Cards),
    #[error("game {0} is already complete")]
    GameComplete(GameId),
    #[error("game {0} has already started")]
    GameHasStarted(GameId),
    #[error("hearts cannot be led if hearts are not broken")]
    HeartsNotBroken,
    #[error("cannot {0}, current phase is {1:?}")]
    IllegalAction(&'static str, GamePhase),
    #[error("{0} is not a legal pass, passes must have 3 cards")]
    IllegalPassSize(Cards),
    #[error("{0} is not a valid name for a human player")]
    InvalidName(String),
    #[error("{0} is not a member of game {1}")]
    InvalidPlayer(String, GameId),
    #[error("charged cards cannot be played on the first trick of their suit")]
    NoChargeOnFirstTrickOfSuit,
    #[error("points cannot be played on the first trick")]
    NoPointsOnFirstTrick,
    #[error("{0} is not claiming, or their claim has been rejected")]
    NotClaiming(String),
    #[error("your hand does not contain {0}")]
    NotYourCards(Cards),
    #[error("player {0} makes the next {1}")]
    NotYourTurn(String, &'static str),
    #[error("api endpoints require a \"name\" cookie identifying the caller")]
    MissingNameCookie,
    #[error("on the first trick if you have nothing but points, you must play the jack of diamonds if you have it")]
    MustPlayJackOfDiamonds,
    #[error("on the first trick if you have nothing but positive points, you must play the queen of spades if you have it")]
    MustPlayQueenOfSpades,
    #[error("the first lead must be the two of clubs")]
    MustPlayTwoOfClubs,
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
    #[error("{0} is not a known game id")]
    UnknownGame(GameId),
    #[error("{0} is not a known player")]
    UnknownPlayer(String),
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

pub async fn handle_rejection(err: Rejection) -> Result<Box<dyn Reply>, Infallible> {
    Ok(if let Some(auth_flow) = err.find::<AuthFlow>() {
        Box::new(auth_flow.to_reply())
    } else if let Some(error) = err.find::<CardsError>() {
        Box::new(
            Response::builder()
                .status(error.status_code())
                .body(error.to_string())
                .unwrap(),
        )
    } else if err.is_not_found() {
        Box::new(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("")
                .unwrap(),
        )
    } else {
        Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("{:?}", err))
                .unwrap(),
        )
    })
}
