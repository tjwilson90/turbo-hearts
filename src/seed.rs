use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

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
