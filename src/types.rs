use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::{Debug, Display},
    str::FromStr,
};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GameId(Uuid);

impl GameId {
    pub fn new() -> GameId {
        GameId(Uuid::new_v4())
    }
}

impl Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChargingRules {
    Classic,
    Blind,
    Bridge,
    BlindBridge,
    Chain,
    BlindChain,
}

impl ChargingRules {
    pub const VALUES: [ChargingRules; 6] = [
        ChargingRules::Classic,
        ChargingRules::Blind,
        ChargingRules::Bridge,
        ChargingRules::BlindBridge,
        ChargingRules::Chain,
        ChargingRules::BlindChain,
    ];

    pub fn free(&self) -> bool {
        match self {
            ChargingRules::Classic | ChargingRules::Blind => true,
            _ => false,
        }
    }

    pub fn chain(&self) -> bool {
        match self {
            ChargingRules::Chain | ChargingRules::BlindChain => true,
            _ => false,
        }
    }

    pub fn blind(&self) -> bool {
        match self {
            ChargingRules::Classic | ChargingRules::Bridge | ChargingRules::Chain => false,
            _ => true,
        }
    }
}

impl Display for ChargingRules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(&self, f)
    }
}

impl ToSql for ChargingRules {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for ChargingRules {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(value) => Ok(ChargingRules::VALUES[value as usize]),
            Err(e) => Err(e),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Seat {
    North,
    East,
    South,
    West,
}

impl Seat {
    pub const VALUES: [Seat; 4] = [Seat::North, Seat::East, Seat::South, Seat::West];

    pub fn idx(&self) -> usize {
        *self as usize
    }

    pub fn next(&self) -> Self {
        match self {
            Seat::North => Seat::East,
            Seat::East => Seat::South,
            Seat::South => Seat::West,
            Seat::West => Seat::North,
        }
    }

    pub fn pass_receiver(&self, hand: Hand) -> Self {
        match hand {
            Hand::Left => self.next(),
            Hand::Right => self.next().next().next(),
            Hand::Across => self.next().next(),
            Hand::Keeper => *self,
        }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl ToSql for Seat {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Hand {
    Left,
    Right,
    Across,
    Keeper,
}

impl Hand {
    pub const VALUES: [Hand; 4] = [Hand::Left, Hand::Right, Hand::Across, Hand::Keeper];

    pub fn idx(&self) -> usize {
        *self as usize
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl ToSql for Hand {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl FromSql for Hand {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        match value.as_i64() {
            Ok(value) => Ok(Hand::VALUES[value as usize]),
            Err(e) => Err(e),
        }
    }
}
