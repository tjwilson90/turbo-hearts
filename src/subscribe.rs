use crate::cards::{Card, Cards, ChargingRules, EventId, GameId, Player};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub game_id: GameId,
    pub event_id: EventId,
    pub player: Player,
    pub timestamp: u64,
    pub event: EventKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EventKind {
    Sit(Player),
    ChargingRules(ChargingRules),
    ReceiveHand(Cards),
    ReceivePass(Cards),
    ChargeCards(Cards),
    BlindCharge(u64),
    PlayCard(Card),
}

impl ToSql for EventKind {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(self).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl FromSql for EventKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| serde_json::from_str(s).unwrap())
    }
}
