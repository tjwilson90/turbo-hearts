use crate::{Cards, GameId, GamePhase, PassDirection, UserId};
use rusqlite::ErrorCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CardsError {
    #[error("{0} has already accepted the claim from {1}")]
    AlreadyAcceptedClaim(UserId, UserId),
    #[error("{0} has already been charged")]
    AlreadyCharged(Cards),
    #[error("{0} has already made a claim")]
    AlreadyClaiming(UserId),
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
    #[error("The {1} hand in game {0} hasn't completed yet")]
    IncompleteHand(GameId, PassDirection),
    #[error("{0} is not a member of game {1}")]
    InvalidPlayer(UserId, GameId),
    #[error("charged cards cannot be played on the first trick of their suit")]
    NoChargeOnFirstTrickOfSuit,
    #[error("points cannot be played on the first trick")]
    NoPointsOnFirstTrick,
    #[error("{0} is not claiming, or their claim has been rejected")]
    NotClaiming(UserId),
    #[error("Games need at least 4 players to start")]
    NotEnoughPlayers,
    #[error("your hand does not contain {0}")]
    NotYourCards(Cards),
    #[error("player {0} makes the next {1}")]
    NotYourTurn(UserId, &'static str),
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
    #[error("{0} is not a known auth token")]
    UnknownAuthToken(String),
    #[error("{0} is not a known game id")]
    UnknownGame(GameId),
    #[error("Either {0} is not a known game id, or the {1} hand hasn't started yet")]
    UnknownHand(GameId, PassDirection),
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
}
