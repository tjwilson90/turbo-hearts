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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Participant {
    pub player: Player,
    pub rules: ChargingRules,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Player {
    Human { name: String },
    Bot { name: String, algorithm: String },
}

impl Player {
    pub fn name(&self) -> &str {
        match self {
            Player::Human { name } => name,
            Player::Bot { name, .. } => name,
        }
    }
}

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ChargingRules {
    Classic,
    Blind,
    Bridge,
    BlindBridge,
    Chain,
    BlindChain,
}

impl ChargingRules {
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

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Seat {
    North,
    East,
    South,
    West,
}

impl Seat {
    pub const VALUES: [Seat; 4] = [Seat::North, Seat::East, Seat::South, Seat::West];

    pub fn all<F>(f: F) -> bool
    where
        F: Fn(Seat) -> bool,
    {
        f(Seat::North) && f(Seat::East) && f(Seat::South) && f(Seat::West)
    }

    pub fn idx(&self) -> usize {
        *self as usize
    }

    pub fn left(&self) -> Self {
        match self {
            Seat::North => Seat::East,
            Seat::East => Seat::South,
            Seat::South => Seat::West,
            Seat::West => Seat::North,
        }
    }

    pub fn right(&self) -> Self {
        match self {
            Seat::North => Seat::West,
            Seat::East => Seat::North,
            Seat::South => Seat::East,
            Seat::West => Seat::South,
        }
    }

    pub fn across(&self) -> Self {
        match self {
            Seat::North => Seat::South,
            Seat::East => Seat::West,
            Seat::South => Seat::North,
            Seat::West => Seat::East,
        }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

pub trait Event {
    fn is_ping(&self) -> bool;
}
