use crate::{Cards, GameId, GamePhase, UserId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RulesError {
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
    #[error("hearts cannot be led if hearts are not broken")]
    HeartsNotBroken,
    #[error("cannot {0}, current phase is {1:?}")]
    IllegalAction(&'static str, GamePhase),
    #[error("{0} is not a legal pass, passes must have 3 cards")]
    IllegalPassSize(Cards),
    #[error("charged cards cannot be played on the first trick of their suit")]
    NoChargeOnFirstTrickOfSuit,
    #[error("points cannot be played on the first trick")]
    NoPointsOnFirstTrick,
    #[error("{0} is not claiming, or their claim has been rejected")]
    NotClaiming(UserId),
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
    #[error("the cards {0} cannot be charged")]
    Unchargeable(Cards),
}
