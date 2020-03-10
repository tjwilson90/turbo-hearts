use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::{Debug, Display},
};

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
