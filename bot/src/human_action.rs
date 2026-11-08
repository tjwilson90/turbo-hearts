//! Direct conditional move model learned from successful human decisions.
//! Context changes the relative value of rank, control, voids and special cards.
use crate::{human_components, skilled_context};
use rand::RngExt;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Rank, Suit};

pub const ACTION_CONTEXT: usize = 18;
pub const ACTION_FEATURES: usize = 24;
mod fitted {
    include!("../../assets/models/human-action.rs");
}

pub fn human_action_observation(
    bot: &BotState,
    state: &GameState,
) -> ([f64; ACTION_CONTEXT], Vec<(Card, [f64; ACTION_FEATURES])>) {
    let hand = bot.post_pass_hand - state.played;
    let unseen = Cards::ALL - bot.post_pass_hand - state.played;
    let legal = state.legal_plays(hand);
    let trick = state.current_trick;
    let components = human_components(bot, state);
    let disrupt =
        crate::moon_defense::disruptive_play(bot, state).map_or(components[1], Cards::from);
    let mut context = [0.0; ACTION_CONTEXT];
    context[..16].copy_from_slice(&skilled_context(bot, state));
    context[16] = trick.cards().contains_any(Cards::POINTS | Card::TenClubs) as u8 as f64;
    context[17] = (!trick.is_empty()
        && context[16] == 0.0
        && !trick.cards().contains(trick.suit().with_rank(Rank::Nine))
        && (trick.len() == 3
            || trick.cards().contains(Card::TwoClubs)
            || (trick.suit() == Suit::Spades
                && state.charges.is_charged(Card::QueenSpades)
                && !state.led_suits.contains(Suit::Spades)))) as u8 as f64;
    let actions = legal
        .into_iter()
        .map(|card| {
            let suit = card.suit();
            let own = hand & suit.cards();
            let other = unseen & suit.cards();
            let lead = trick.is_empty();
            let follow = !lead && trick.suit() == suit;
            let mut after = state.clone();
            after.apply(&GameEvent::Play {
                seat: bot.seat,
                card,
            });
            let complete = after.current_trick.is_empty();
            let wins = if complete {
                after.next_actor == Some(bot.seat)
            } else {
                after.current_trick.winning_seat(after.next_actor.unwrap()) == bot.seat
            };
            let points = trick.cards() | card;
            let probability = |choices: Cards| {
                if choices.contains(card) {
                    1.0 / choices.len() as f64
                } else {
                    0.0
                }
            };
            (
                card,
                [
                    card.rank() as u8 as f64 / 12.0,
                    (suit == Suit::Hearts) as u8 as f64,
                    (card == Card::QueenSpades) as u8 as f64,
                    (card == Card::AceHearts) as u8 as f64,
                    (card == Card::JackDiamonds) as u8 as f64,
                    (card == Card::TenClubs) as u8 as f64,
                    (card.rank() == Rank::Nine) as u8 as f64,
                    other.above(card).is_empty() as u8 as f64,
                    (own.max() == card) as u8 as f64,
                    own.len() as f64 / 13.0,
                    (own.len() == 1) as u8 as f64,
                    own.below(card).len() as f64 / 13.0,
                    other.above(card).len() as f64 / 13.0,
                    follow as u8 as f64,
                    lead as u8 as f64,
                    (!lead && !follow) as u8 as f64,
                    wins as u8 as f64,
                    complete as u8 as f64,
                    ((points & Cards::HEARTS).len()
                        + 13 * points.contains(Card::QueenSpades) as usize)
                        as f64
                        / 26.0,
                    (complete && !wins && suit == Suit::Hearts) as u8 as f64,
                    probability(components[1]),
                    probability(components[2]),
                    probability(components[3]),
                    probability(disrupt),
                ],
            )
        })
        .collect();
    (context, actions)
}

pub struct HumanActionPolicy;
impl HumanActionPolicy {
    pub fn distribution(bot: &BotState, state: &GameState) -> Vec<(Card, f64)> {
        let (context, actions) = human_action_observation(bot, state);
        let mut coefficients = [0.0; ACTION_FEATURES];
        for (x, weights) in context.iter().zip(fitted::WEIGHTS) {
            for (c, w) in coefficients.iter_mut().zip(weights) {
                *c += x * w;
            }
        }
        let mut scores: Vec<_> = actions
            .into_iter()
            .map(|(card, x)| {
                (
                    card,
                    x.iter().zip(coefficients).map(|(a, b)| a * b).sum::<f64>(),
                )
            })
            .collect();
        let max = scores.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
        for (_, v) in &mut scores {
            *v = (*v - max).max(-60.0).exp();
        }
        let sum: f64 = scores.iter().map(|s| s.1).sum();
        for (_, v) in &mut scores {
            *v /= sum;
        }
        scores
    }
    pub fn play_with_rng<R: RngExt + ?Sized>(
        bot: &BotState,
        state: &GameState,
        rng: &mut R,
    ) -> Card {
        let distribution = Self::distribution(bot, state);
        let mut target = rng.random::<f64>();
        for &(card, p) in &distribution {
            target -= p;
            if target <= 0.0 {
                return card;
            }
        }
        distribution.last().unwrap().0
    }
}
