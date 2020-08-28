use crate::{BotStrategy, ChargingRules, Seat, UserId};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlayerWithOptions {
    pub player: Player,
    pub rules: ChargingRules,
    pub seat: Option<Seat>,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Player {
    Human {
        user_id: UserId,
    },
    Bot {
        user_id: UserId,
        strategy: BotStrategy,
    },
}

impl Player {
    pub fn user_id(&self) -> UserId {
        match self {
            Player::Human { user_id } => *user_id,
            Player::Bot { user_id, .. } => *user_id,
        }
    }

    pub fn strategy(&self) -> Option<BotStrategy> {
        match self {
            Player::Bot { strategy, .. } => Some(*strategy),
            _ => None,
        }
    }
}
