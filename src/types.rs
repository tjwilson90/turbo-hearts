use num_enum::{IntoPrimitive, TryFromPrimitive};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, fmt::Debug, str::FromStr};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GameId(Uuid);

impl GameId {
    pub fn new() -> GameId {
        GameId(Uuid::new_v4())
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FromStr for GameId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GameId(s.parse()?))
    }
}

impl ToSql for GameId {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Text(self.0.to_string())))
    }
}

impl FromSql for GameId {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_str() {
            Ok(value) => Ok(value.parse().unwrap()),
            Err(e) => Err(e),
        }
    }
}

pub type EventId = u32;

pub type Player = String;

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
pub enum ChargingRules {
    Classic,
    Blind,
    Bridge,
    BlindBridge,
    Chain,
    BlindChain,
}

impl fmt::Display for ChargingRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self, f)
    }
}

impl ToSql for ChargingRules {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(u8::from(*self) as i64)))
    }
}

impl FromSql for ChargingRules {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(value) => Ok(Self::try_from(value as u8).unwrap()),
            Err(e) => Err(e),
        }
    }
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
pub enum Seat {
    North,
    East,
    South,
    West,
}

impl Seat {
    pub const VALUES: [Seat; 4] = [Seat::North, Seat::East, Seat::South, Seat::West];

    pub fn idx(&self) -> usize {
        u8::from(*self) as usize
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl ToSql for Seat {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(u8::from(*self) as i64)))
    }
}

impl FromSql for Seat {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(value) => Ok(Seat::VALUES[value as usize]),
            Err(e) => Err(e),
        }
    }
}

#[repr(u8)]
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
pub enum PassDirection {
    Left,
    Right,
    Across,
    Keeper,
}

impl PassDirection {
    pub fn idx(&self) -> usize {
        u8::from(*self) as usize
    }
}

impl fmt::Display for PassDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl ToSql for PassDirection {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(u8::from(*self) as i64)))
    }
}

impl FromSql for PassDirection {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(value) => Ok(Self::try_from(value as u8).unwrap()),
            Err(e) => Err(e),
        }
    }
}
