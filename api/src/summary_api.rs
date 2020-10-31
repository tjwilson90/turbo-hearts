use crate::{Cards, ChargingRules, GameEvent, GameId, Player, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteGame {
    pub game_id: GameId,
    pub completed_time: i64,
    pub players: [Player; 4],
    pub hands: Vec<CompleteHand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteHand {
    pub charges: [Cards; 4],
    pub hearts_won: [u8; 4],
    pub queen_winner: UserId,
    pub ten_winner: UserId,
    pub jack_winner: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    pub game_id: Option<GameId>,
    pub page_size: Option<u32>,
}

pub type LeaderboardResponse = Vec<CompleteGame>;

#[derive(Debug, Serialize, Deserialize)]
pub struct HandResponse {
    pub north: Player,
    pub east: Player,
    pub south: Player,
    pub west: Player,
    pub rules: ChargingRules,
    pub events: Vec<HandEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HandEvent {
    pub timestamp: i64,
    pub event: GameEvent,
    pub synthetic_events: Vec<GameEvent>,
}
