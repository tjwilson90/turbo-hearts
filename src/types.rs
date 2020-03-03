use crate::{bot::Strategy, seat::Seat, user::UserId};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fmt::{Debug, Display},
};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize)]
pub struct PlayerWithOptions {
    pub player: Player,
    pub rules: ChargingRules,
    pub seat: Option<Seat>,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Player {
    Human { user_id: UserId },
    Bot { user_id: UserId, strategy: Strategy },
}

impl Player {
    pub fn user_id(&self) -> UserId {
        match self {
            Player::Human { user_id } => *user_id,
            Player::Bot { user_id, .. } => *user_id,
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
pub enum PassDirection {
    Left,
    Right,
    Across,
    Keeper,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Seed {
    Chosen { value: String },
    Random { value: String },
    Redacted,
}

impl Seed {
    pub fn random() -> Self {
        Seed::Random {
            value: Uuid::new_v4().to_string(),
        }
    }

    pub fn redact(&self) -> Self {
        match self {
            Seed::Random { .. } => Seed::Redacted,
            _ => self.clone(),
        }
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        Sha256::digest(match self {
            Seed::Chosen { value } => value.as_bytes(),
            Seed::Random { value } => value.as_bytes(),
            Seed::Redacted => panic!("cannot convert redacted seed to bytes"),
        })
        .into()
    }
}

pub enum RandomEvent {
    Deal(PassDirection),
    KeeperPass,
}

impl RandomEvent {
    fn id(&self) -> u64 {
        match self {
            RandomEvent::Deal(PassDirection::Left) => 0,
            RandomEvent::Deal(PassDirection::Right) => 1,
            RandomEvent::Deal(PassDirection::Across) => 2,
            RandomEvent::Deal(PassDirection::Keeper) => 3,
            RandomEvent::KeeperPass => 4,
        }
    }

    pub fn rng(&self, seed: [u8; 32]) -> ChaCha20Rng {
        let mut rng = ChaCha20Rng::from_seed(seed);
        rng.set_stream(self.id());
        rng
    }
}

pub trait Event {
    fn is_ping(&self) -> bool;
}
