use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{export::Formatter, Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    fmt::{Debug, Display},
    str::FromStr,
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

impl PassDirection {
    pub fn rng(&self, seed: [u8; 32]) -> ChaCha20Rng {
        let mut rng = ChaCha20Rng::from_seed(seed);
        rng.set_stream(*self as u8 as u64);
        rng
    }
}
