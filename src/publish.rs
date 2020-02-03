use crate::cards::{Card, Cards, ChargingRules, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub game_id: u64,
    pub player: Player,
    pub event: EventKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventKind {
    ChargingRules(ChargingRules),
    PassCards(Cards),
    ChargeCards(Cards),
    PlayCard(Card),
}
