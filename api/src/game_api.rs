use crate::{Card, Cards, GameId, Seat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PassRequest {
    pub game_id: GameId,
    pub cards: Cards,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChargeRequest {
    pub game_id: GameId,
    pub cards: Cards,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayRequest {
    pub game_id: GameId,
    pub card: Card,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub game_id: GameId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptClaimRequest {
    pub game_id: GameId,
    pub claimer: Seat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RejectClaimRequest {
    pub game_id: GameId,
    pub claimer: Seat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameChatRequest {
    pub game_id: GameId,
    pub message: String,
}
