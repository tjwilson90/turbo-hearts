use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};
use uuid::Uuid;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(&self, f)
    }
}

sql_json!(ChargingRules);

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

sql_str!(GameId);

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum PassDirection {
    Left,
    Right,
    Across,
    Keeper,
}

impl Display for PassDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(&self, f)
    }
}

impl FromStr for PassDirection {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(PassDirection::Left),
            "right" => Ok(PassDirection::Right),
            "across" => Ok(PassDirection::Across),
            "keeper" => Ok(PassDirection::Keeper),
            _ => panic!("Invalid pass direction {}", s),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn null() -> UserId {
        UserId(Uuid::nil())
    }

    pub fn new() -> UserId {
        UserId(Uuid::new_v4())
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        Display::fmt(&self.0, f)
    }
}

impl FromStr for UserId {
    type Err = <Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UserId(s.parse()?))
    }
}

sql_str!(UserId);
