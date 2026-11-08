//! A small contextual mixture fitted to historical human actions. Its scores
//! are action probabilities, never monetary priors or perfect-information play.
use crate::{DuckBot, GottaTryBot, HeuristicBot};
use rand::RngExt;
use turbo_hearts_api::{BotState, Card, Cards, GameState, Seat};

pub const HUMAN_CONTEXT: usize = 8;
pub const HUMAN_COMPONENTS: usize = 5;

mod fitted {
    include!("../../assets/models/human-policy.rs");
}

pub fn human_context(bot: &BotState, state: &GameState) -> [f64; HUMAN_CONTEXT] {
    let hand = bot.post_pass_hand - state.played;
    let hearts = hand & Cards::HEARTS;
    let unseen = Cards::HEARTS - hand - state.played;
    [
        1.0,
        state.won.can_run(bot.seat) as u8 as f64,
        state.won.hearts(bot.seat) as f64 / 13.0,
        state.won.queen(bot.seat) as u8 as f64,
        Seat::VALUES.iter().any(|&s| {
            s != bot.seat && state.won.can_run(s) && (state.won.hearts(s) > 0 || state.won.queen(s))
        }) as u8 as f64,
        hearts.len() as f64 / hand.len().max(1) as f64,
        (!hearts.is_empty() && unseen.above(hearts.max()).is_empty()) as u8 as f64,
        state.played.len() as f64 / 52.0,
    ]
}

pub fn human_components(bot: &BotState, state: &GameState) -> [Cards; HUMAN_COMPONENTS] {
    let heuristic = HeuristicBot.choices(bot, state);
    [
        state.legal_plays(bot.post_pass_hand),
        heuristic,
        DuckBot.choices(bot, state),
        GottaTryBot.choices(bot, state),
        crate::moon_defense::planned_play(bot, state).map_or(heuristic, Cards::from),
    ]
}

/// Component likelihoods for one observed legal action, for offline fitting.
pub fn human_likelihoods(bot: &BotState, state: &GameState, card: Card) -> [f64; HUMAN_COMPONENTS] {
    human_components(bot, state).map(|cards| {
        if cards.contains(card) {
            1.0 / cards.len() as f64
        } else {
            0.0
        }
    })
}

pub struct HumanPolicy;

impl HumanPolicy {
    pub fn weights(bot: &BotState, state: &GameState) -> [f64; HUMAN_COMPONENTS] {
        let x = human_context(bot, state);
        let logits = fitted::WEIGHTS.map(|w| w.iter().zip(x).map(|(a, b)| a * b).sum::<f64>());
        let max = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let weights = logits.map(|v| (v - max).exp());
        let sum: f64 = weights.iter().sum();
        weights.map(|v| v / sum)
    }

    pub fn probability(bot: &BotState, state: &GameState, card: Card) -> f64 {
        Self::weights(bot, state)
            .iter()
            .zip(human_likelihoods(bot, state, card))
            .map(|(a, b)| a * b)
            .sum()
    }

    pub fn play_with_rng<R: RngExt + ?Sized>(
        bot: &BotState,
        state: &GameState,
        rng: &mut R,
    ) -> Card {
        let mut target = rng.random::<f64>();
        let weights = Self::weights(bot, state);
        let mut component = HUMAN_COMPONENTS - 1;
        for (i, weight) in weights.iter().enumerate() {
            target -= weight;
            if target <= 0.0 {
                component = i;
                break;
            }
        }
        let choices = human_components(bot, state)[component];
        choices
            .into_iter()
            .nth(rng.random_range(0..choices.len()))
            .unwrap()
    }
}
