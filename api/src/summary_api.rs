use crate::{Cards, ChargingRules, GameEvent, GameId, Player, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    pub game_id: Option<GameId>,
    pub page_size: Option<u32>,
}

pub type LeaderboardResponse = Vec<LeaderboardGame>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardGame {
    pub game_id: GameId,
    pub completed_time: i64,
    pub players: [Player; 4],
    pub hands: Vec<LeaderboardHand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardHand {
    pub charges: [Cards; 4],
    pub hearts_won: [u8; 4],
    pub queen_winner: UserId,
    pub ten_winner: UserId,
    pub jack_winner: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameEventsRequest {
    pub game_id: Option<GameId>,
    pub page_size: Option<u32>,
}

pub type GameEventsResponse = Vec<GameSummaryResponse>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSummaryResponse {
    pub game_id: GameId,
    pub players: [Player; 4],
    pub rules: ChargingRules,
    pub hands: Vec<Vec<GameSummaryEvent>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameSummaryEvent {
    pub timestamp: i64,
    pub event: GameEvent,
    pub synthetic_events: Vec<GameEvent>,
}
