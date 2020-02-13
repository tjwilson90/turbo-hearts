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

pub type Name = String;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Participant {
    pub player: Player,
    pub rules: ChargingRules,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Player {
    Human { name: Name },
    Bot { name: Name, algorithm: String },
}

impl Player {
    pub fn name(&self) -> &Name {
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

    pub fn pass_sender(&self, hand: PassDirection) -> Self {
        match hand {
            PassDirection::Left => self.right(),
            PassDirection::Right => self.left(),
            PassDirection::Across => self.across(),
            PassDirection::Keeper => *self,
        }
    }

    pub fn pass_receiver(&self, hand: PassDirection) -> Self {
        match hand {
            PassDirection::Left => self.left(),
            PassDirection::Right => self.right(),
            PassDirection::Across => self.across(),
            PassDirection::Keeper => *self,
        }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PassDirection {
    Left,
    Right,
    Across,
    Keeper,
}

impl PassDirection {
    pub fn next(self) -> Option<PassDirection> {
        match self {
            PassDirection::Left => Some(PassDirection::Right),
            PassDirection::Right => Some(PassDirection::Across),
            PassDirection::Across => Some(PassDirection::Keeper),
            PassDirection::Keeper => None,
        }
    }

    pub fn first_charger(self) -> Seat {
        match self {
            PassDirection::Left => Seat::North,
            PassDirection::Right => Seat::East,
            PassDirection::Across => Seat::South,
            PassDirection::Keeper => Seat::West,
        }
    }
}

pub trait Event {
    fn is_ping(&self) -> bool;
}
