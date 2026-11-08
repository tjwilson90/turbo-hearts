//! Actor-only values for passing and charging, fitted to responsive terminal money.
use std::{convert::TryInto, fs, io, path::Path};
use turbo_hearts_api::{BotState, Cards, GamePhase, GameState, Rank, Suit};

pub const PREPLAY_FEATURES: usize = 256;
pub const PREPLAY_HIDDEN: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreplayKind {
    Pass,
    Charge,
}

#[derive(Clone)]
pub struct PreplayNetwork {
    kind: PreplayKind,
    first: Vec<f32>,
    bias: [f32; PREPLAY_HIDDEN],
    output: [f32; PREPLAY_HIDDEN],
    intercept: f32,
}

impl PreplayNetwork {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let bad = || io::Error::new(io::ErrorKind::InvalidData, "invalid preplay checkpoint");
        let count = PREPLAY_FEATURES * PREPLAY_HIDDEN + 2 * PREPLAY_HIDDEN + 1;
        if bytes.len() != 16 + 4 * count || &bytes[..4] != b"TPQ1" {
            return Err(bad());
        }
        let dim = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
        if dim(4) != PREPLAY_FEATURES as u32 || dim(8) != PREPLAY_HIDDEN as u32 || dim(12) > 1 {
            return Err(bad());
        }
        let values: Vec<f32> = bytes[16..]
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        if values.iter().any(|v| !v.is_finite()) {
            return Err(bad());
        }
        let n = PREPLAY_FEATURES * PREPLAY_HIDDEN;
        let mut bias = [0.0; PREPLAY_HIDDEN];
        bias.copy_from_slice(&values[n..n + PREPLAY_HIDDEN]);
        let mut output = [0.0; PREPLAY_HIDDEN];
        output.copy_from_slice(&values[n + PREPLAY_HIDDEN..n + 2 * PREPLAY_HIDDEN]);
        Ok(Self {
            kind: if dim(12) == 0 {
                PreplayKind::Pass
            } else {
                PreplayKind::Charge
            },
            first: values[..n].to_vec(),
            bias,
            output,
            intercept: values[count - 1],
        })
    }
    pub fn kind(&self) -> PreplayKind {
        self.kind
    }
    pub fn predict(&self, x: &[f32; PREPLAY_FEATURES]) -> f32 {
        let mut h = self.bias;
        for (&x, weights) in x.iter().zip(self.first.chunks_exact(PREPLAY_HIDDEN)) {
            if x != 0.0 {
                for (h, w) in h.iter_mut().zip(weights) {
                    *h += x * w;
                }
            }
        }
        100.0
            * h.iter()
                .zip(self.output)
                .fold(self.intercept, |sum, (h, w)| sum + h.max(0.0) * w)
    }
    pub fn choose(&self, bot: &BotState, state: &GameState, baseline: Cards, margin: f32) -> Cards {
        let base = self.predict(&preplay_features(bot, state, baseline, self.kind));
        let mut best = (baseline, base + margin);
        for action in preplay_actions(bot, state, self.kind) {
            let value = self.predict(&preplay_features(bot, state, action, self.kind));
            if value > best.1 {
                best = (action, value);
            }
        }
        best.0
    }
}

pub fn preplay_actions(bot: &BotState, state: &GameState, kind: PreplayKind) -> Vec<Cards> {
    match kind {
        PreplayKind::Pass => bot.pre_pass_hand.choose(3).collect(),
        PreplayKind::Charge => ((bot.post_pass_hand & Cards::CHARGEABLE)
            - state.charges.all_charges())
        .powerset()
        .collect(),
    }
}

/// TPQ1 layout: three card channels; direction/task/rules/stage; relative
/// public charges/turn/declines; original/retained suit summaries.
/// No other player's hand, intended pass, blind charge, or eventual score enters.
pub fn preplay_features(
    bot: &BotState,
    state: &GameState,
    action: Cards,
    kind: PreplayKind,
) -> [f32; PREPLAY_FEATURES] {
    let hand = if kind == PreplayKind::Pass {
        bot.pre_pass_hand
    } else {
        bot.post_pass_hand
    };
    let kept = if kind == PreplayKind::Pass {
        hand - action
    } else {
        hand
    };
    let mut x = [0.0; PREPLAY_FEATURES];
    for (channel, cards) in [hand, kept, action].iter().copied().enumerate() {
        for card in cards {
            x[channel * 52 + card.suit().idx() * 13 + card.rank().idx()] = 1.0;
        }
    }
    x[156 + state.phase.direction() as usize] = 1.0;
    x[160 + kind as usize] = 1.0;
    x[162] = (state.phase == GamePhase::ChargeKeeper1) as u8 as f32;
    x[163] = (state.phase == GamePhase::ChargeKeeper2) as u8 as f32;
    x[164] = state.rules.free() as u8 as f32;
    x[165] = state.rules.blind() as u8 as f32;
    x[166] = state.rules.chain() as u8 as f32;
    x[167] = state.charge_count as f32 / 4.0;
    for (rel, seat) in [
        bot.seat,
        bot.seat.left(),
        bot.seat.across(),
        bot.seat.right(),
    ]
    .iter()
    .copied()
    .enumerate()
    {
        for card in state.charges.charges(seat) {
            x[168 + rel * 4 + card.suit().idx()] = 1.0;
        }
    }
    for (rel, seat) in [
        bot.seat,
        bot.seat.left(),
        bot.seat.across(),
        bot.seat.right(),
    ]
    .iter()
    .copied()
    .enumerate()
    {
        x[184 + rel] = (state.next_actor == Some(seat)) as u8 as f32;
        x[252 + rel] = state.done.charged(seat) as u8 as f32;
    }
    for (channel, cards) in [hand, kept].iter().copied().enumerate() {
        for suit in Suit::VALUES {
            let own = cards & suit.cards();
            let offset = 188 + channel * 32 + suit.idx() * 8;
            x[offset] = own.len() as f32 / 13.0;
            x[offset + 1] = own.below(suit.with_rank(Rank::Nine)).len() as f32 / 13.0;
            x[offset + 2] = own.above(suit.with_rank(Rank::Ten)).len() as f32 / 13.0;
            let mut controls = 0;
            for card in suit.cards() {
                if own.contains(card) {
                    controls += 1;
                } else {
                    break;
                }
            }
            x[offset + 3] = controls as f32 / 13.0;
            x[offset + 4] = own.contains(suit.with_rank(Rank::Nine)) as u8 as f32;
            x[offset + 5] = own.contains_any(Cards::CHARGEABLE) as u8 as f32;
            x[offset + 6] = if own.is_empty() {
                0.0
            } else {
                own.min().rank().idx() as f32 / 12.0
            };
            x[offset + 7] = own.into_iter().map(|c| c.rank().idx() as f32).sum::<f32>() / 156.0;
        }
    }
    x
}
