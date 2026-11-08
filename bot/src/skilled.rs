//! Contextual action model learned from consistently successful human players.
//! Used as a continuation policy, never as a monetary value estimate.
use crate::{human_context, DuckBot, GottaTryBot, HeuristicBot};
use rand::RngExt;
use turbo_hearts_api::{BotState, Card, Cards, GameState, Rank, Suit};

pub const SKILLED_CONTEXT: usize = 16;
pub const SKILLED_COMPONENTS: usize = 6;
mod fitted {
    include!("../../assets/models/skilled-policy.rs");
}

pub fn skilled_context(bot: &BotState, state: &GameState) -> [f64; SKILLED_CONTEXT] {
    let hand = bot.post_pass_hand - state.played;
    let unseen = Cards::ALL - bot.post_pass_hand - state.played;
    let mut x = [0.0; SKILLED_CONTEXT];
    x[..8].copy_from_slice(&human_context(bot, state));
    x[8] = hand.contains(Card::QueenSpades) as u8 as f64;
    x[9] = state.charges.is_charged(Card::QueenSpades) as u8 as f64;
    x[11] = (hand & Cards::NINES).len() as f64 / 4.0;
    for suit in Suit::VALUES {
        let own = hand & suit.cards();
        let other = unseen & suit.cards();
        if own.is_empty() {
            x[14] += 0.25;
            continue;
        }
        x[13] = x[13].max(own.len() as f64 / hand.len().max(1) as f64);
        let master = other.above(own.max()).is_empty();
        x[10] += master as u8 as f64 / 4.0;
        x[12] += (master && own.contains(suit.with_rank(Rank::Nine))) as u8 as f64 / 4.0;
        if !other.is_empty() {
            x[15] += own.below(other.max()).len() as f64 / hand.len().max(1) as f64;
        }
    }
    x
}

pub fn skilled_components(bot: &BotState, state: &GameState) -> [Cards; SKILLED_COMPONENTS] {
    std::array::from_fn(|index| component_choices(bot, state, index))
}

pub(crate) fn component_choices(bot: &BotState, state: &GameState, index: usize) -> Cards {
    match index {
        0 => state.legal_plays(bot.post_pass_hand),
        1 => HeuristicBot.choices(bot, state),
        2 => DuckBot.choices(bot, state),
        3 => GottaTryBot.choices(bot, state),
        4 => crate::moon_defense::planned_play(bot, state)
            .map(Cards::from)
            .unwrap_or_else(|| HeuristicBot.choices(bot, state)),
        5 => crate::moon_defense::disruptive_play(bot, state)
            .map(Cards::from)
            .unwrap_or_else(|| HeuristicBot.choices(bot, state)),
        _ => unreachable!(),
    }
}

pub fn skilled_likelihoods(
    bot: &BotState,
    state: &GameState,
    card: Card,
) -> [f64; SKILLED_COMPONENTS] {
    skilled_components(bot, state).map(|cards| {
        if cards.contains(card) {
            1.0 / cards.len() as f64
        } else {
            0.0
        }
    })
}

pub struct SkilledPolicy;
impl SkilledPolicy {
    pub fn weights(bot: &BotState, state: &GameState) -> [f64; SKILLED_COMPONENTS] {
        let x = skilled_context(bot, state);
        let logits = fitted::WEIGHTS.map(|w| w.iter().zip(x).map(|(a, b)| a * b).sum::<f64>());
        let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let weights = logits.map(|v| (v - max).exp());
        let total: f64 = weights.iter().sum();
        weights.map(|v| v / total)
    }
    pub fn probability(bot: &BotState, state: &GameState, card: Card) -> f64 {
        Self::weights(bot, state)
            .iter()
            .zip(skilled_likelihoods(bot, state, card))
            .map(|(w, p)| w * p)
            .sum()
    }
    pub fn play_with_rng<R: RngExt + ?Sized>(
        bot: &BotState,
        state: &GameState,
        rng: &mut R,
    ) -> Card {
        let mut target = rng.random::<f64>();
        let legal = state.legal_plays(bot.post_pass_hand);
        if legal.len() == 1 {
            // Keep both draws made by the mixture/component path, including
            // random_range's implementation for a singleton choice.
            let _ = rng.random_range(0..1usize);
            return legal.max();
        }
        let weights = Self::weights(bot, state);
        let mut component = SKILLED_COMPONENTS - 1;
        for (i, p) in weights.iter().enumerate() {
            target -= p;
            if target <= 0.0 {
                component = i;
                break;
            }
        }
        // Only evaluate the sampled component; constructing all six policies
        // at every simulated move adds work without changing the distribution.
        let choices = component_choices(bot, state, component);
        choices
            .into_iter()
            .nth(rng.random_range(0..choices.len()))
            .unwrap()
    }
}
