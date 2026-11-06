//! A small action-value network trained from sampled search, using only the
//! acting player's information. The compact format avoids an inference runtime.
use std::{convert::TryInto, fs, io, path::Path};
use turbo_hearts_api::{BotState, Card, Cards, GameState, Rank, Suit};

pub const POLICY_FEATURES: usize = 256;
const HIDDEN: usize = 64;

#[derive(Clone)]
pub struct PolicyNetwork {
    weights: Vec<f32>,
}

impl PolicyNetwork {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let size = POLICY_FEATURES * HIDDEN + HIDDEN + HIDDEN + 1;
        if bytes.len() != 12 + 4 * size
            || &bytes[..4] != b"THP1"
            || u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize != POLICY_FEATURES
            || u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize != HIDDEN
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid policy header or dimensions",
            ));
        }
        let weights: Vec<f32> = bytes[12..]
            .chunks_exact(4)
            .map(|v| f32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        if weights.iter().any(|v| !v.is_finite()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "nonfinite policy weights",
            ));
        }
        Ok(Self { weights })
    }

    /// Returns the scalar output multiplied by 100. Distilled value checkpoints
    /// use money advantage; policy-gradient checkpoints use policy scores and
    /// must not be interpreted as calibrated monetary values.
    pub fn predict(&self, input: &[f32; POLICY_FEATURES]) -> f32 {
        let first_bias = POLICY_FEATURES * HIDDEN;
        let output = first_bias + HIDDEN;
        let mut value = self.weights[output + HIDDEN];
        for h in 0..HIDDEN {
            let weights = &self.weights[h * POLICY_FEATURES..(h + 1) * POLICY_FEATURES];
            let activation = input
                .iter()
                .zip(weights)
                .fold(self.weights[first_bias + h], |sum, (x, w)| sum + x * w)
                .max(0.0);
            value += self.weights[output + h] * activation;
        }
        value * 100.0
    }
}

/// Versioned by THP1. No sampled opponent cards enter this representation.
/// Seats are relative to the actor, so rotating a deal leaves features unchanged.
pub fn policy_features(bot: &BotState, state: &GameState, card: Card) -> [f32; POLICY_FEATURES] {
    let hand = bot.post_pass_hand - state.played;
    let unseen = Cards::ALL - state.played - hand;
    let trick = state.current_trick.cards();
    let mut values = Vec::with_capacity(POLICY_FEATURES);
    for cards in [hand, unseen, trick] {
        for c in Cards::ALL {
            values.push(cards.contains(c) as u8 as f32);
        }
    }
    for s in Suit::VALUES {
        values.push((s == card.suit()) as u8 as f32);
    }
    for r in 0..13 {
        values.push((r == card.rank().idx()) as u8 as f32);
    }
    let seat = bot.seat;
    let seats = [seat, seat.left(), seat.across(), seat.right()];
    for s in seats {
        values.extend_from_slice(&[
            state.won.hearts(s) as f32 / 13.0,
            state.won.queen(s) as u8 as f32,
            state.won.jack(s) as u8 as f32,
            state.won.ten(s) as u8 as f32,
        ]);
    }
    for c in Cards::CHARGEABLE {
        values.push(state.charges.is_charged(c) as u8 as f32);
    }
    for s in Suit::VALUES {
        values.push(state.led_suits.contains(s) as u8 as f32);
    }
    for s in seats {
        for suit in Suit::VALUES {
            values.push(bot.void.is_void(s, suit) as u8 as f32);
        }
    }
    for s in seats {
        values
            .push((!trick.is_empty() && state.current_trick.winning_seat(seat) == s) as u8 as f32);
    }
    for s in Suit::VALUES {
        values.push((!trick.is_empty() && state.current_trick.suit() == s) as u8 as f32);
    }
    values.extend_from_slice(&[trick.len() as f32 / 8.0, state.played.len() as f32 / 52.0]);
    // Rank and suit summaries make small-data training substantially easier.
    let ours = hand & card.suit().cards();
    let theirs = unseen & card.suit().cards();
    values.extend_from_slice(&[
        ours.len() as f32 / 13.0,
        theirs.len() as f32 / 13.0,
        ours.above(card).len() as f32 / 13.0,
        ours.below(card).len() as f32 / 13.0,
        theirs.above(card).len() as f32 / 13.0,
        theirs.below(card).len() as f32 / 13.0,
        (card.rank() == Rank::Nine) as u8 as f32,
        state.charges.is_charged(card) as u8 as f32,
    ]);
    for suit in Suit::VALUES {
        values.push((hand & suit.cards()).len() as f32 / 13.0);
        values.push((unseen & suit.cards()).len() as f32 / 13.0);
    }
    for s in seats {
        values.push(state.won.can_run(s) as u8 as f32);
    }
    let mut after = state.clone();
    after.apply(&turbo_hearts_api::GameEvent::Play { seat, card });
    values.push(after.current_trick.is_empty() as u8 as f32);
    for s in seats {
        values.push(
            (if after.current_trick.is_empty() {
                after.next_actor.unwrap()
            } else {
                after.current_trick.winning_seat(after.next_actor.unwrap())
            } == s) as u8 as f32,
        );
    }
    assert_eq!(
        values.len(),
        248,
        "changing features requires a new model format version"
    );
    let mut result = [0.0; POLICY_FEATURES];
    result[..values.len()].copy_from_slice(&values);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_models_match_numpy_reference() {
        let cases: [(&[u8], f32); 2] = [
            (
                include_bytes!("../../assets/models/policy-avoid.thp"),
                0.6544297337532043,
            ),
            (
                include_bytes!("../../assets/models/policy-moon.thp"),
                0.2817147672176361,
            ),
        ];
        for (bytes, expected) in cases {
            let network = PolicyNetwork::from_bytes(bytes).unwrap();
            let input = std::array::from_fn(|i| (i % 7) as f32 / 7.0);
            // Independent NumPy forward pass over the exported float32 weights.
            assert!((network.predict(&input) - expected).abs() < 0.001);
        }
    }

    #[test]
    fn validates_format_and_matches_dense_layer_arithmetic() {
        assert!(PolicyNetwork::from_bytes(b"bad").is_err());
        let count = POLICY_FEATURES * HIDDEN + 2 * HIDDEN + 1;
        let mut weights = vec![0f32; count];
        weights[0] = 2.0;
        weights[POLICY_FEATURES * HIDDEN] = -1.0;
        weights[POLICY_FEATURES * HIDDEN + HIDDEN] = 3.0;
        weights[count - 1] = 0.5;
        let mut bytes = b"THP1".to_vec();
        bytes.extend_from_slice(&(POLICY_FEATURES as u32).to_le_bytes());
        bytes.extend_from_slice(&(HIDDEN as u32).to_le_bytes());
        for w in weights {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        let network = PolicyNetwork::from_bytes(&bytes).unwrap();
        let mut x = [0.0; POLICY_FEATURES];
        x[0] = 2.0;
        assert_eq!(network.predict(&x), 950.0);
        x[0] = 0.0;
        assert_eq!(network.predict(&x), 50.0);
        bytes[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(PolicyNetwork::from_bytes(&bytes).is_err());
    }
}
