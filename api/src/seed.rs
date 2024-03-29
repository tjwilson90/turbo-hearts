use crate::{Cards, GameEvent, PassDirection, Seat};
use log::info;
use rand::{seq::SliceRandom, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::BorrowMut;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
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
}

#[derive(Clone, Debug)]
pub struct HashedSeed {
    seed: [u8; 32],
}

impl From<&Seed> for HashedSeed {
    fn from(seed: &Seed) -> Self {
        Self {
            seed: Sha256::digest(match seed {
                Seed::Chosen { value } => value.as_bytes(),
                Seed::Random { value } => value.as_bytes(),
                Seed::Redacted => panic!("cannot convert redacted seed to bytes"),
            })
            .into(),
        }
    }
}

impl HashedSeed {
    pub fn new() -> Self {
        Self { seed: [0; 32] }
    }

    pub fn deal(&self, pass: PassDirection) -> GameEvent {
        let mut rng = ChaCha20Rng::from_seed(self.seed);
        rng.set_stream(pass as u64);
        let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
        deck.shuffle(&mut rng);
        let north = deck[0..13].iter().cloned().collect();
        let east = deck[13..26].iter().cloned().collect();
        let south = deck[26..39].iter().cloned().collect();
        let west = deck[39..52].iter().cloned().collect();
        info!(
            "deal: north={}, east={}, south={}, west={}, pass={}",
            north, east, south, west, pass
        );
        GameEvent::Deal {
            north,
            east,
            south,
            west,
            pass,
        }
    }

    pub fn keeper_pass(&self, partial_hands: [Cards; 4]) -> [GameEvent; 4] {
        let mut rng = ChaCha20Rng::from_seed(self.seed);
        rng.set_stream(4);
        let cards =
            Cards::ALL - partial_hands[0] - partial_hands[1] - partial_hands[2] - partial_hands[3];
        let mut passes: Vec<_> = cards.into_iter().collect();
        passes.shuffle(&mut rng);
        let mut passes = passes.into_iter();
        [
            GameEvent::RecvPass {
                to: Seat::North,
                cards: passes
                    .borrow_mut()
                    .take(13 - partial_hands[0].len())
                    .collect(),
            },
            GameEvent::RecvPass {
                to: Seat::East,
                cards: passes
                    .borrow_mut()
                    .take(13 - partial_hands[1].len())
                    .collect(),
            },
            GameEvent::RecvPass {
                to: Seat::South,
                cards: passes
                    .borrow_mut()
                    .take(13 - partial_hands[2].len())
                    .collect(),
            },
            GameEvent::RecvPass {
                to: Seat::West,
                cards: passes.collect(),
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deal_left() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        assert_eq!(
            seed.deal(PassDirection::Left),
            GameEvent::Deal {
                north: "QJT7S KJ643H J97D AC".parse().unwrap(),
                east: "AK962S Q9H 82D QJ63C".parse().unwrap(),
                south: "53S A52H 54D T97542C".parse().unwrap(),
                west: "84S T87H AKQT63D K8C".parse().unwrap(),
                pass: PassDirection::Left
            }
        );
    }

    #[test]
    fn test_deal_left_then_right() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        seed.deal(PassDirection::Left);
        assert_eq!(
            seed.deal(PassDirection::Right),
            GameEvent::Deal {
                north: "A3S KQT542H Q9642C".parse().unwrap(),
                east: "JT62S A976H AQ5D A8C".parse().unwrap(),
                south: "K987S JH KJ7643D J7C".parse().unwrap(),
                west: "Q54S 83H T982D KT53C".parse().unwrap(),
                pass: PassDirection::Right
            }
        );
    }

    #[test]
    fn test_deal_right() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        assert_eq!(
            seed.deal(PassDirection::Right),
            GameEvent::Deal {
                north: "A3S KQT542H Q9642C".parse().unwrap(),
                east: "JT62S A976H AQ5D A8C".parse().unwrap(),
                south: "K987S JH KJ7643D J7C".parse().unwrap(),
                west: "Q54S 83H T982D KT53C".parse().unwrap(),
                pass: PassDirection::Right
            }
        );
    }

    #[test]
    fn test_deal_across() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        assert_eq!(
            seed.deal(PassDirection::Across),
            GameEvent::Deal {
                north: "KT9S 76H QJ75D Q743C".parse().unwrap(),
                east: "6532S Q952H K2D J86C".parse().unwrap(),
                south: "QJ8S K83H AT6D AKT9C".parse().unwrap(),
                west: "A74S AJT4H 9843D 52C".parse().unwrap(),
                pass: PassDirection::Across
            }
        );
    }

    #[test]
    fn test_deal_keeper() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        assert_eq!(
            seed.deal(PassDirection::Keeper),
            GameEvent::Deal {
                north: "3S A852H 96432D KQ3C".parse().unwrap(),
                east: "AKQJ742S KD AT864C".parse().unwrap(),
                south: "T98S K973H AQT7D 72C".parse().unwrap(),
                west: "65S QJT64H J85D J95C".parse().unwrap(),
                pass: PassDirection::Keeper
            }
        );
    }

    #[test]
    fn test_keeper_pass() {
        let seed = HashedSeed::from(&Seed::Chosen {
            value: "chosen".to_string(),
        });
        assert_eq!(
            seed.keeper_pass([
                "AK9865432S AH".parse().unwrap(),
                "QT987652H AKD".parse().unwrap(),
                "QT865432D KQC".parse().unwrap(),
                "JT98765432C".parse().unwrap()
            ]),
            [
                GameEvent::RecvPass {
                    to: Seat::North,
                    cards: "TS KH 9D".parse().unwrap()
                },
                GameEvent::RecvPass {
                    to: Seat::East,
                    cards: "J4H AC".parse().unwrap()
                },
                GameEvent::RecvPass {
                    to: Seat::South,
                    cards: "JS J7D".parse().unwrap()
                },
                GameEvent::RecvPass {
                    to: Seat::West,
                    cards: "Q7S 3H".parse().unwrap()
                }
            ]
        );
    }
}
