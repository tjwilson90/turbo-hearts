//! Small history-conditioned policies and action advantages. All inputs belong
//! to the actor; private passes observed by another seat are never propagated.
use crate::human_action_observation;
use rand::RngExt;
use std::{convert::TryInto, fs, io, path::Path};
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Rank, Seat, Suit};

pub const HUMAN_INPUTS: usize = 363;
pub const HUMAN_ACTIONS: usize = 28;
pub const HUMAN_OUTPUTS: usize = HUMAN_ACTIONS + 52;
pub const HUMAN_ROW: usize = 6 + HUMAN_INPUTS + 13 * (1 + HUMAN_ACTIONS);

#[derive(Clone, Debug)]
pub struct HumanMemory {
    observer: Option<Seat>,
    passed: [Cards; 4],
    pass_known: bool,
    public: [[f32; 8]; 4],
}
impl Default for HumanMemory {
    fn default() -> Self {
        Self {
            observer: None,
            passed: [Cards::NONE; 4],
            pass_known: false,
            public: [[0.0; 8]; 4],
        }
    }
}
impl HumanMemory {
    pub fn on_event(&mut self, own: &BotState, state: &GameState, event: &GameEvent) {
        if matches!(event, GameEvent::Deal { .. }) {
            *self = Self::default();
            self.observer = Some(own.seat);
        }
        if let GameEvent::SendPass { from, cards } = event {
            if *from == own.seat {
                let to = state.phase.pass_receiver(*from);
                if to != *from {
                    self.passed[to.idx()] |= *cards;
                    self.pass_known = true;
                }
            }
        }
        self.observe_public(state, event);
    }
    pub fn observe_public(&mut self, state: &GameState, event: &GameEvent) {
        if let GameEvent::Play { seat, card } = event {
            let stats = &mut self.public[seat.idx()];
            let trick = state.current_trick;
            let rank = card.rank().idx() as f32 / 12.0;
            stats[0] += 1.0 / 13.0;
            stats[7] = rank;
            if !trick.is_empty() {
                let follow = card.suit() == trick.suit();
                let safe = state.played.len() < 24
                    && follow
                    && card.suit() != Suit::Hearts
                    && !trick.cards().contains_any(Cards::POINTS | Card::TenClubs)
                    && !trick.cards().contains(trick.suit().with_rank(Rank::Nine))
                    && (trick.len() == 3
                        || trick.cards().contains(Card::TwoClubs)
                        || (trick.suit() == Suit::Spades
                            && state.charges.is_charged(Card::QueenSpades)
                            && !state.led_suits.contains(Suit::Spades)));
                if safe {
                    stats[1] += 1.0 / 6.0;
                    stats[2] += rank / 6.0;
                    stats[3] += (card.rank() < Rank::Nine) as u8 as f32 / 6.0;
                    stats[4] += (card.rank() >= Rank::King) as u8 as f32 / 6.0;
                }
                if !follow {
                    stats[5] += (card.suit() == Suit::Hearts) as u8 as f32 / 3.0;
                    stats[6] += (card.rank() == Rank::Nine) as u8 as f32 / 2.0;
                }
            }
        }
    }
    pub fn without_private(&self) -> Self {
        let mut result = self.clone();
        result.observer = None;
        result.passed = [Cards::NONE; 4];
        result.pass_known = false;
        result
    }
}

pub fn human_card_index(card: Card) -> usize {
    card.suit().idx() * 13 + card.rank().idx()
}

pub fn neural_human_observation(
    bot: &BotState,
    state: &GameState,
    memory: &HumanMemory,
) -> ([f32; HUMAN_INPUTS], Vec<(Card, [f32; HUMAN_ACTIONS])>) {
    observation(bot, state, memory, true)
}

fn observation(
    bot: &BotState,
    state: &GameState,
    memory: &HumanMemory,
    rich: bool,
) -> ([f32; HUMAN_INPUTS], Vec<(Card, [f32; HUMAN_ACTIONS])>) {
    let (old, actions) = human_action_observation(bot, state);
    let mut x = [0.0; HUMAN_INPUTS];
    for (to, from) in x.iter_mut().zip(old) {
        *to = from as f32;
    }
    if rich {
        let hand = bot.post_pass_hand - state.played;
        for (channel, cards) in [
            hand,
            Cards::ALL - bot.post_pass_hand - state.played,
            state.current_trick.cards(),
        ]
        .iter()
        .copied()
        .enumerate()
        {
            for card in cards {
                x[18 + channel * 52 + human_card_index(card)] = 1.0;
            }
        }
        if memory.observer == Some(bot.seat) && memory.pass_known {
            for (rel, seat) in [bot.seat.left(), bot.seat.across(), bot.seat.right()]
                .iter()
                .copied()
                .enumerate()
            {
                for card in memory.passed[seat.idx()] - state.played {
                    x[174 + rel * 52 + human_card_index(card)] = 1.0;
                }
            }
            x[362] = 1.0;
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
            x[330 + rel * 8..338 + rel * 8].copy_from_slice(&memory.public[seat.idx()]);
        }
    }
    let actions = actions
        .into_iter()
        .map(|(card, old)| {
            let mut a = [0.0; HUMAN_ACTIONS];
            for (to, from) in a.iter_mut().zip(old) {
                *to = from as f32;
            }
            a[24 + card.suit().idx()] = 1.0;
            (card, a)
        })
        .collect();
    (x, actions)
}

#[derive(Clone)]
pub struct HumanNetwork {
    inputs: usize,
    hidden: usize,
    weights: Vec<f32>,
    first: Vec<f32>,
    active_outputs: usize,
    pub(crate) advantage: bool,
}
impl HumanNetwork {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::from_bytes(&fs::read(path)?)
    }
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid human neural checkpoint",
            )
        };
        if bytes.len() < 20 || &bytes[..4] != b"THH1" {
            return Err(invalid());
        }
        let u = |i| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        let (inputs, hidden, outputs, role) = (u(4), u(8), u(12), u(16));
        if ![18, HUMAN_INPUTS].contains(&inputs)
            || ![32, 64].contains(&hidden)
            || outputs != HUMAN_OUTPUTS
            || role > 1
            || bytes.len() != 20 + 4 * (inputs * hidden + hidden + hidden * outputs + outputs)
        {
            return Err(invalid());
        }
        let weights: Vec<_> = bytes[20..]
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        if weights.iter().any(|v| !v.is_finite()) {
            return Err(invalid());
        }
        let mut first = vec![0.0; inputs * hidden];
        for h in 0..hidden {
            for i in 0..inputs {
                first[i * hidden + h] = weights[h * inputs + i];
            }
        }
        // Capacity controls fix the unused card/suit outputs to zero. Avoid
        // evaluating those columns without changing any nonzero accumulation.
        let second = inputs * hidden + hidden;
        let bias = second + hidden * HUMAN_OUTPUTS;
        let active_outputs = (0..HUMAN_OUTPUTS)
            .rfind(|&output| {
                weights[bias + output] != 0.0
                    || (0..hidden).any(|h| weights[second + h * HUMAN_OUTPUTS + output] != 0.0)
            })
            .map_or(0, |output| output + 1);
        Ok(Self {
            inputs,
            hidden,
            weights,
            first,
            active_outputs,
            advantage: role == 1,
        })
    }
    pub fn coefficients(&self, x: &[f32; HUMAN_INPUTS]) -> [f32; HUMAN_OUTPUTS] {
        let first = self.inputs * self.hidden;
        let second = first + self.hidden;
        let bias = second + self.hidden * HUMAN_OUTPUTS;
        let mut result = [0.0; HUMAN_OUTPUTS];
        result.copy_from_slice(&self.weights[bias..]);
        let mut hidden = [0.0f32; 64];
        hidden[..self.hidden].copy_from_slice(&self.weights[first..second]);
        // Input-major weights permit vectorization across independent units;
        // skip zero card/history features. Accumulation order stays unchanged.
        for (&value, w) in x[..self.inputs]
            .iter()
            .zip(self.first.chunks_exact(self.hidden))
        {
            if value != 0.0 {
                for (a, w) in hidden[..self.hidden].iter_mut().zip(w) {
                    *a += value * w;
                }
            }
        }
        for (&activation, w) in hidden[..self.hidden]
            .iter()
            .zip(self.weights[second..bias].chunks_exact(HUMAN_OUTPUTS))
        {
            let a = activation.max(0.0);
            if a != 0.0 {
                for (out, w) in result[..self.active_outputs].iter_mut().zip(w) {
                    *out += a * w;
                }
            }
        }
        result
    }
    pub fn scores(
        &self,
        bot: &BotState,
        state: &GameState,
        memory: &HumanMemory,
    ) -> Vec<(Card, f32)> {
        let (x, actions) = observation(bot, state, memory, self.inputs == HUMAN_INPUTS);
        let c = self.coefficients(&x);
        actions
            .into_iter()
            .map(|(card, a)| {
                (
                    card,
                    c[HUMAN_ACTIONS + human_card_index(card)]
                        + a.iter().zip(c).map(|(a, b)| a * b).sum::<f32>(),
                )
            })
            .collect()
    }
    pub fn distribution(
        &self,
        bot: &BotState,
        state: &GameState,
        memory: &HumanMemory,
    ) -> Vec<(Card, f64)> {
        assert!(
            !self.advantage,
            "payoff heads are not human action probabilities"
        );
        let scores = self.scores(bot, state, memory);
        let max = scores.iter().map(|v| v.1).fold(f32::NEG_INFINITY, f32::max);
        let mut result: Vec<_> = scores
            .into_iter()
            .map(|(c, v)| (c, ((v - max) as f64).exp()))
            .collect();
        let sum: f64 = result.iter().map(|v| v.1).sum();
        for (_, p) in &mut result {
            *p /= sum;
        }
        result
    }
    pub fn play<R: RngExt + ?Sized>(
        &self,
        bot: &BotState,
        state: &GameState,
        memory: &HumanMemory,
        rng: &mut R,
    ) -> Card {
        assert!(
            !self.advantage,
            "payoff heads are not human action probabilities"
        );
        let mut t = rng.random::<f64>();
        let legal = state.legal_plays(bot.post_pass_hand);
        // Preserve the random draw even for forced moves, so paired rollout
        // streams and outcomes remain unchanged by this optimization.
        if legal.len() == 1 {
            return legal.max();
        }
        let p = self.distribution(bot, state, memory);
        for &(c, v) in &p {
            t -= v;
            if t <= 0.0 {
                return c;
            }
        }
        p.last().unwrap().0
    }
}
