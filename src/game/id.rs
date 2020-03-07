use serde::{Deserialize, Serialize};
use std::{fmt, fmt::Display, str::FromStr};
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
