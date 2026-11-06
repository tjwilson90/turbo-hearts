//! Pass/charge search using complete scoring, including moons and keeper redraws.
use crate::experimental::simulate_style;
use crate::{Algorithm, DuckBot, GottaTryBot, HeuristicBot, NeuralNetworkBot};
use rand::{seq::SliceRandom, Rng};
use std::time::Instant;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Seat, VoidState};

fn style_pass(style: usize, seat: Seat, hand: Cards, state: &GameState) -> Cards {
    let bot = BotState::new(seat, hand);
    match style {
        1 => DuckBot.pass(&bot, state),
        2 => GottaTryBot.pass(&bot, state),
        _ => HeuristicBot.pass(&bot, state),
    }
}

fn finish_preplay<R: Rng + ?Sized>(
    state: &mut GameState,
    hands: &mut [Cards; 4],
    root: Seat,
    own_charge: usize,
    styles: [usize; 4],
    rng: &mut R,
) {
    let mut turns = 0;
    while !state.phase.is_playing() {
        if state.phase.is_passing() {
            // This path is only reached after nobody charges the keeper hand.
            let passes: [Cards; 4] =
                std::array::from_fn(|i| style_pass(styles[i], Seat::VALUES[i], hands[i], state));
            let mut pool = Vec::new();
            for seat in Seat::VALUES {
                hands[seat.idx()] -= passes[seat.idx()];
                pool.extend(passes[seat.idx()]);
                state.apply(&GameEvent::SendPass {
                    from: seat,
                    cards: passes[seat.idx()],
                });
            }
            pool.shuffle(rng);
            let mut cards = pool.into_iter();
            for seat in Seat::VALUES {
                let pass = cards
                    .by_ref()
                    .take(13 - hands[seat.idx()].len())
                    .collect::<Cards>();
                hands[seat.idx()] |= pass;
                state.apply(&GameEvent::RecvPass {
                    to: seat,
                    cards: pass,
                });
            }
        } else {
            for seat in Seat::VALUES {
                if !state.phase.is_charging() {
                    break;
                }
                if !state.can_charge(seat) || state.done.charged(seat) {
                    continue;
                }
                let bot = BotState::new(seat, hands[seat.idx()]);
                let cards = if seat == root {
                    match own_charge {
                        0 => Cards::NONE,
                        2 => (bot.post_pass_hand & Cards::CHARGEABLE) - state.charges.all_charges(),
                        _ => HeuristicBot.charge_with_rng(&bot, state, rng),
                    }
                } else {
                    match styles[seat.idx()] {
                        1 => Cards::NONE,
                        2 => GottaTryBot.charge(&bot, state),
                        _ => HeuristicBot.charge_with_rng(&bot, state, rng),
                    }
                };
                state.apply(&GameEvent::Charge { seat, cards });
                turns += 1;
                assert!(turns < 64, "preplay did not terminate");
            }
        }
    }
    state.next_actor = Seat::VALUES
        .iter()
        .copied()
        .find(|s| hands[s.idx()].contains(Card::TwoClubs));
}

impl NeuralNetworkBot {
    pub(crate) fn search_pass(&mut self, bot: &BotState, state: &GameState) -> Cards {
        let start = Instant::now();
        let bases = [
            HeuristicBot.pass(bot, state),
            GottaTryBot.pass(bot, state),
            DuckBot.pass(bot, state),
        ];
        let mut candidates = Vec::new();
        for base in bases {
            if !candidates.contains(&base) {
                candidates.push(base);
            }
            for outgoing in base {
                for incoming in bot.pre_pass_hand - base {
                    let candidate = (base - outgoing) | incoming;
                    if !candidates.contains(&candidate) {
                        candidates.push(candidate);
                    }
                }
            }
        }
        let mut totals = vec![[0.0f64; 6]; candidates.len()];
        let mut sampler = self.hand_maker.sampler(bot.void);
        // Pre-play experiments use a separate stream, preserving paired play RNGs.
        let mut rng = self.rng.clone();
        for sample in 0..self.experiments.pass_samples {
            if sample > 0
                && self
                    .config
                    .time_limit
                    .map_or(false, |limit| start.elapsed() >= limit)
            {
                break;
            }
            let hands = sampler.sample(&mut rng);
            let mut styles = std::array::from_fn(|_| {
                let x = rng.gen::<f32>();
                if x < 0.7 {
                    0
                } else if x < 0.85 {
                    1
                } else {
                    2
                }
            });
            styles[bot.seat.idx()] = 0;
            let passes: [Cards; 4] =
                std::array::from_fn(|i| style_pass(styles[i], Seat::VALUES[i], hands[i], state));
            let seed: u64 = rng.gen();
            for (i, &candidate) in candidates.iter().enumerate() {
                let mut passes = passes;
                passes[bot.seat.idx()] = candidate;
                let mut post = hands;
                for seat in Seat::VALUES {
                    post[seat.idx()] -= passes[seat.idx()];
                    post[state.phase.pass_receiver(seat).idx()] |= passes[seat.idx()];
                }
                for charge in 0..3 {
                    let mut game = GameState::new();
                    game.rules = state.rules;
                    game.phase = state.phase.next(false);
                    game.next_actor = game.phase.first_charger(game.rules);
                    let mut hands = post;
                    let mut sim_rng =
                        <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(seed);
                    finish_preplay(
                        &mut game,
                        &mut hands,
                        bot.seat,
                        charge,
                        styles,
                        &mut sim_rng,
                    );
                    for shoot in 0..2 {
                        let mut outcome = game.clone();
                        let mut sim_rng = sim_rng.clone();
                        simulate_style(
                            &mut outcome,
                            VoidState::new(),
                            hands,
                            bot.seat,
                            shoot == 1,
                            Some(styles),
                            &self.experiments,
                            &mut sim_rng,
                        );
                        totals[i][2 * charge + shoot] += outcome.scores().money(bot.seat) as f64;
                    }
                }
            }
        }
        candidates
            .into_iter()
            .zip(totals)
            .max_by(|a, b| {
                let best =
                    |values: [f64; 6]| values.iter().fold(f64::NEG_INFINITY, |a, b| a.max(*b));
                best(a.1).partial_cmp(&best(b.1)).unwrap()
            })
            .unwrap()
            .0
    }

    pub(crate) fn search_charge(&mut self, bot: &BotState, state: &GameState) -> Cards {
        let start = Instant::now();
        let available = (bot.post_pass_hand & Cards::CHARGEABLE) - state.charges.all_charges();
        if available.is_empty() {
            return Cards::NONE;
        }
        let candidates: Vec<_> = available.powerset().collect();
        let mut totals = vec![[0.0f64; 2]; candidates.len()];
        let mut sampler = self.hand_maker.sampler(bot.void);
        let mut rng = self.rng.clone();
        for sample in 0..self.experiments.charge_samples {
            if sample > 0
                && self
                    .config
                    .time_limit
                    .map_or(false, |limit| start.elapsed() >= limit)
            {
                break;
            }
            let hands = sampler.sample(&mut rng);
            let mut styles = std::array::from_fn(|_| {
                let x = rng.gen::<f32>();
                if x < 0.7 {
                    0
                } else if x < 0.85 {
                    1
                } else {
                    2
                }
            });
            styles[bot.seat.idx()] = 0;
            let seed: u64 = rng.gen();
            for (i, &cards) in candidates.iter().enumerate() {
                let mut game = state.clone();
                let mut hands = hands;
                game.apply(&GameEvent::Charge {
                    seat: bot.seat,
                    cards,
                });
                let mut sim_rng = <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(seed);
                finish_preplay(&mut game, &mut hands, bot.seat, 0, styles, &mut sim_rng);
                for shoot in 0..2 {
                    let mut game = game.clone();
                    let mut sim_rng = sim_rng.clone();
                    simulate_style(
                        &mut game,
                        VoidState::new(),
                        hands,
                        bot.seat,
                        shoot == 1,
                        Some(styles),
                        &self.experiments,
                        &mut sim_rng,
                    );
                    totals[i][shoot] += game.scores().money(bot.seat) as f64;
                }
            }
        }
        candidates
            .into_iter()
            .zip(totals)
            .max_by(|a, b| a.1[0].max(a.1[1]).partial_cmp(&b.1[0].max(b.1[1])).unwrap())
            .unwrap()
            .0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use turbo_hearts_api::{ChargingRules, GamePhase};

    #[test]
    fn preplay_preserves_ownership_and_terminates_under_all_charging_rules() {
        let mut rng = StdRng::seed_from_u64(771);
        for rules in [
            ChargingRules::Classic,
            ChargingRules::Blind,
            ChargingRules::Bridge,
            ChargingRules::BlindBridge,
            ChargingRules::Chain,
            ChargingRules::BlindChain,
        ] {
            for phase in [
                GamePhase::ChargeLeft,
                GamePhase::ChargeRight,
                GamePhase::ChargeAcross,
                GamePhase::ChargeKeeper1,
            ] {
                for charge in 0..3 {
                    let mut deck: Vec<_> = Cards::ALL.into_iter().collect();
                    deck.shuffle(&mut rng);
                    let mut hands: [Cards; 4] = std::array::from_fn(|i| {
                        deck[13 * i..13 * (i + 1)].iter().copied().collect()
                    });
                    let mut state = GameState::new();
                    state.phase = phase;
                    state.rules = rules;
                    state.next_actor = phase.first_charger(rules);
                    finish_preplay(
                        &mut state,
                        &mut hands,
                        Seat::North,
                        charge,
                        [0, 1, 1, 1],
                        &mut rng,
                    );
                    assert!(state.phase.is_playing());
                    let mut seen = Cards::NONE;
                    for seat in Seat::VALUES {
                        assert_eq!(hands[seat.idx()].len(), 13);
                        assert!(!seen.contains_any(hands[seat.idx()]));
                        seen |= hands[seat.idx()];
                    }
                    assert_eq!(seen, Cards::ALL);
                    assert!(hands[state.next_actor.unwrap().idx()].contains(Card::TwoClubs));
                    if charge == 0 {
                        assert!(state.charges.all_charges().is_empty());
                    }
                }
            }
        }
    }
}
