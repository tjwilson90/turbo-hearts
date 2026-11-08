//! Reproducible opt-in research variants. Production settings are selected by
//! held-out matches, not by a promising training loss or a single lineup.
use crate::neural_network::{rollout, solve_endgame};
use crate::{policy_features, DuckBot, GottaTryBot, HeuristicBot, NeuralNetworkBot};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{collections::HashMap, time::Instant};
use turbo_hearts_api::{
    BotState, Card, Cards, GameEvent, GameState, Rank, Seat, Suit, VoidState, WonState,
};

/// Search extensions. `default()` disables all extensions for frozen-baseline
/// experiments; `production()` selects the subset confirmed in fresh matches.
#[derive(Clone, Debug)]
pub struct SearchExperiments {
    pub opponent_model: bool,
    /// Adopt non-heuristic styles only after strong within-hand evidence;
    /// otherwise retain the frozen baseline's opposing-shooter mixture.
    pub conservative_model: bool,
    pub style_confidence: f64,
    /// Once a moon is impossible, duck and GottaTry have the same play policy.
    pub collapse_equivalent_styles: bool,
    /// Fraction of otherwise unmodeled players that defend an opposing moon.
    pub moon_defense: f64,
    /// Value a shooting continuation only if two independent sample halves
    /// both prefer it to avoidance. This adds no rollout evaluations.
    pub moon_consensus: bool,
    /// Reassess emerging opponent moon threats during a continuation.
    pub dynamic_shooters: bool,
    /// Compare a separate own continuation that preserves and uses moon stoppers.
    pub defensive_plan: bool,
    /// Recognize provable claims at trick boundaries during continuations.
    pub claim_rollouts: bool,
    /// Allow the fallback shooter mixture after the first captured heart.
    pub early_shooters: bool,
    /// Maximum modeled money sacrificed to retain a supported master heart.
    pub stopper_margin: f32,
    /// Fraction of opponent continuations sampled from the fitted human model.
    pub human_rollouts: f64,
    /// Use the successful-human model for the fitted fraction of continuations.
    pub skilled_model: bool,
    /// Use the direct successful-human conditional move model.
    pub action_model: bool,
    /// Treat the fitted human policy as a fallback: retain an inferred duck or
    /// shooter style, including sampled early shooting evidence.
    pub human_model_fallback: bool,
    /// Tempered first-opportunity human charging likelihood, all four cards.
    pub charge_belief_strength: f64,
    /// Soft early shooting evidence from declining safe high-card disposal.
    pub early_intent: bool,
    /// Add an own continuation which distributes hearts and removes nines.
    pub disruptive_plan: bool,
    /// Compare a separate own continuation from the successful-human move model.
    pub learned_plan: bool,
    pub cache_endgames: bool,
    pub canonical_endgames: bool,
    /// Power applied to the behavioral likelihood; zero keeps uniform deals.
    pub belief_strength: f64,
    /// Number of subsequent own decisions explored in an observation tree.
    pub tree_depth: usize,
    pub allocate: bool,
    /// Minimum shared rounds before successively halving root candidates.
    pub allocation_min: usize,
    pub pass_samples: usize,
    pub charge_samples: usize,
}

impl Default for SearchExperiments {
    fn default() -> Self {
        Self {
            opponent_model: false,
            conservative_model: false,
            style_confidence: 0.9,
            collapse_equivalent_styles: false,
            moon_defense: 0.0,
            moon_consensus: false,
            dynamic_shooters: false,
            defensive_plan: false,
            claim_rollouts: false,
            early_shooters: false,
            stopper_margin: 0.0,
            human_rollouts: 0.0,
            skilled_model: false,
            action_model: false,
            human_model_fallback: false,
            charge_belief_strength: 0.0,
            early_intent: false,
            disruptive_plan: false,
            learned_plan: false,
            cache_endgames: false,
            canonical_endgames: false,
            belief_strength: 0.0,
            tree_depth: 0,
            allocate: false,
            allocation_min: 32,
            pass_samples: 0,
            charge_samples: 0,
        }
    }
}

impl SearchExperiments {
    /// Default play settings selected by human-focused evaluations.
    pub fn production() -> Self {
        Self {
            human_rollouts: 1.0,
            skilled_model: true,
            human_model_fallback: true,
            ..Self::round_four()
        }
    }

    /// Frozen supported-stopper policy, before successful-human modeling.
    pub fn round_four() -> Self {
        Self {
            stopper_margin: 8.0,
            ..Self::round_three()
        }
    }

    /// Frozen November round-three baseline for reproducible comparisons.
    pub fn round_three() -> Self {
        Self {
            conservative_model: true,
            style_confidence: 0.6,
            dynamic_shooters: true,
            ..Self::default()
        }
    }

    pub(crate) fn changes_play(&self) -> bool {
        self.opponent_model
            || self.conservative_model
            || self.moon_defense > 0.0
            || self.moon_consensus
            || self.dynamic_shooters
            || self.defensive_plan
            || self.claim_rollouts
            || self.early_shooters
            || self.human_rollouts > 0.0
            || self.charge_belief_strength > 0.0
            || self.early_intent
            || self.disruptive_plan
            || self.learned_plan
            || self.belief_strength > 0.0
            || self.tree_depth > 0
            || self.allocate
    }
}

#[derive(Clone)]
struct Observation {
    state: GameState,
    seat: Seat,
    card: Card,
    void: VoidState,
}

#[derive(Clone)]
pub(crate) struct BehaviorHistory {
    plays: Vec<Observation>,
    void: VoidState,
}

impl Default for BehaviorHistory {
    fn default() -> Self {
        Self {
            plays: Vec::new(),
            void: VoidState::new(),
        }
    }
}

impl BehaviorHistory {
    fn early_runner_probability(&self, hands: [Cards; 4], seat: Seat) -> f64 {
        let mut evidence = 0.0;
        let mut opportunities = 0;
        for obs in self.plays.iter().filter(|o| o.seat == seat) {
            let bot = BotState::with_void(seat, hands[seat.idx()], obs.void);
            if let Some(log_ratio) = control_signal(&bot, &obs.state, obs.card) {
                evidence += log_ratio;
                opportunities += 1;
            }
        }
        if opportunities == 0 {
            return 0.0;
        }
        // Heuristic likelihood, not a fitted probability of human intent.
        // Temper correlated observations and retain uncertainty in either direction.
        let log_odds = (0.1_f64 / 0.9).ln() + 0.5 * evidence;
        (1.0 / (1.0 + (-log_odds).exp())).min(0.75)
    }
    fn could_adapt(&self, root: Seat) -> bool {
        Seat::VALUES
            .iter()
            .any(|&seat| seat != root && self.plays.iter().filter(|o| o.seat == seat).count() >= 3)
    }
    fn decisions(&self, hands: [Cards; 4], seat: Seat) -> usize {
        self.plays
            .iter()
            .filter(|o| o.seat == seat && o.state.legal_plays(hands[seat.idx()]).len() > 1)
            .count()
    }
    pub(crate) fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Deal { .. } | GameEvent::HandComplete { .. } => *self = Self::default(),
            GameEvent::Play { seat, card } => {
                self.plays.push(Observation {
                    state: state.clone(),
                    seat: *seat,
                    card: *card,
                    void: self.void,
                });
                self.void = self.void.on_event(state, event);
            }
            _ => {}
        }
    }

    fn log_likelihoods(&self, hands: [Cards; 4], root: Seat) -> [[f64; 3]; 4] {
        let mut result = [[0.0; 3]; 4];
        for obs in &self.plays {
            if obs.seat == root {
                continue;
            }
            let bot = BotState::with_void(obs.seat, hands[obs.seat.idx()], obs.void);
            let legal = obs.state.legal_plays(bot.post_pass_hand);
            if legal.len() <= 1 {
                continue;
            }
            let choices = [
                HeuristicBot.choices(&bot, &obs.state),
                DuckBot.choices(&bot, &obs.state),
                GottaTryBot.choices(&bot, &obs.state),
            ];
            for p in 0..3 {
                // The noise component accounts for humans/search bots that do
                // not exactly follow one of these three approximate policies.
                let probability = 0.25 / legal.len() as f64
                    + if choices[p].contains(obs.card) {
                        0.75 / choices[p].len() as f64
                    } else {
                        0.0
                    };
                result[obs.seat.idx()][p] += probability.ln();
            }
        }
        result
    }
}

/// Compare low-preserving-control and high-unloading-card rank distributions
/// only at early, non-forced follow-suit opportunities with a legal K/A.
fn control_signal(bot: &BotState, state: &GameState, played: Card) -> Option<f64> {
    let trick = state.current_trick;
    if trick.is_empty()
        || state.played.len() > 24
        || !state.won.can_run(bot.seat)
        || trick.cards().contains_any(Cards::POINTS | Card::TenClubs)
        || trick.suit() == Suit::Hearts
        || trick.cards().contains(trick.suit().with_rank(Rank::Nine))
    {
        return None;
    }
    let legal = state.legal_plays(bot.post_pass_hand);
    if legal.len() < 2
        || !trick.suit().cards().contains_all(legal)
        || legal.max().rank() < Rank::King
        || legal.min().rank() >= Rank::King
    {
        return None;
    }
    let safe = trick.cards().contains(Card::TwoClubs)
        || trick.len() == 3
        || (trick.suit() == Suit::Spades
            && state.charges.is_charged(Card::QueenSpades)
            && !state.led_suits.contains(Suit::Spades));
    if !safe {
        return None;
    }
    let cards: Vec<_> = legal.into_iter().collect();
    let index = cards.iter().position(|&c| c == played)?;
    // Cards iterate from high to low. Normalize both distributions over the
    // actor's legal alternatives; forced lows supply no evidence.
    let low: Vec<_> = (0..cards.len())
        .map(|i| (2.0 * i as f64 / (cards.len() - 1) as f64).exp())
        .collect();
    let high: Vec<_> = low.iter().rev().copied().collect();
    let probability =
        |w: &[f64]| 0.25 / cards.len() as f64 + 0.75 * w[index] / w.iter().sum::<f64>();
    Some((probability(&low) / probability(&high)).ln())
}

fn posterior_with_prior(logs: [f64; 3], prior: [f64; 3]) -> ([f64; 3], f64) {
    let values: [f64; 3] = std::array::from_fn(|i| prior[i].ln() + 0.5 * logs[i]);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights = values.map(|v| (v - max).exp());
    let sum: f64 = weights.iter().sum();
    (weights.map(|v| v / sum), max + sum.ln())
}

fn posterior(logs: [f64; 3]) -> ([f64; 3], f64) {
    posterior_with_prior(logs, [0.6, 0.2, 0.2])
}

fn choose_weighted<R: RngExt + ?Sized>(weights: &[f64], rng: &mut R) -> usize {
    let sum: f64 = weights.iter().sum();
    let mut target = rng.random::<f64>() * sum;
    for (i, weight) in weights.iter().enumerate() {
        target -= weight;
        if target <= 0.0 {
            return i;
        }
    }
    weights.len() - 1
}

struct World {
    hands: [Cards; 4],
    styles: [[f64; 3]; 4],
    log_weight: f64,
    charge_log_weight: f64,
    runners: [f64; 4],
}

impl NeuralNetworkBot {
    /// Select experiments before replaying game history, including Sit, so
    /// inference receives public player types and the complete hand history.
    pub fn with_experiments(mut self, settings: SearchExperiments) -> Self {
        assert!(
            settings.charge_belief_strength.is_finite()
                && (0.0..=1.0).contains(&settings.charge_belief_strength)
        );
        assert!(
            settings.belief_strength.is_finite() && (0.0..=1.0).contains(&settings.belief_strength)
        );
        assert!(settings.tree_depth <= 4);
        assert!(
            settings.human_rollouts.is_finite() && (0.0..=1.0).contains(&settings.human_rollouts)
        );
        assert!(settings.stopper_margin.is_finite() && settings.stopper_margin >= 0.0);
        assert!(settings.moon_defense.is_finite() && (0.0..=1.0).contains(&settings.moon_defense));
        assert!(
            settings.style_confidence.is_finite()
                && (0.5..=1.0).contains(&settings.style_confidence)
        );
        self.experiments = settings;
        self
    }

    pub(crate) fn experimental_search(
        &mut self,
        bot: &BotState,
        state: &GameState,
    ) -> (Vec<(Card, f32)>, usize) {
        let start = Instant::now();
        let plays: Vec<_> = state
            .legal_plays(bot.post_pass_hand)
            .distinct_plays(state.played, state.current_trick)
            .into_iter()
            .collect();
        let mut sampler = self.hand_maker.sampler(bot.void);
        let conservative = self.experiments.conservative_model;
        let exact = 52 - state.played.len() <= self.config.exact_cards;
        let charge_beliefs = self.experiments.charge_belief_strength > 0.0
            && self.charge_history.has_evidence(bot.seat);
        let infer = self.experiments.opponent_model
            || (conservative && !exact && self.history.could_adapt(bot.seat))
            || self.experiments.belief_strength > 0.0
            || charge_beliefs
            || (self.experiments.early_intent && !self.history.plays.is_empty());
        let mut inference_rng = self.rng.clone();
        let mut pool = Vec::new();
        if infer {
            for _ in 0..self.config.samples.max(32) * 2 {
                let hands = sampler.sample(if conservative {
                    &mut inference_rng
                } else {
                    &mut self.rng
                });
                let logs = self.history.log_likelihoods(hands, bot.seat);
                let results = logs.map(|log| {
                    if conservative {
                        posterior_with_prior(log, [0.9, 0.05, 0.05])
                    } else {
                        posterior(log)
                    }
                });
                pool.push(World {
                    hands,
                    charge_log_weight: if charge_beliefs {
                        self.charge_history.log_likelihood(hands, bot.seat)
                    } else {
                        0.0
                    },
                    runners: Seat::VALUES.map(|s| {
                        if self.experiments.early_intent && s != bot.seat {
                            self.history.early_runner_probability(hands, s)
                        } else {
                            0.0
                        }
                    }),
                    styles: results.map(|r| r.0),
                    log_weight: results
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != bot.seat.idx())
                        .map(|(_, r)| r.1)
                        .sum(),
                });
            }
        }
        let log_weight = |w: &World| {
            w.log_weight * self.experiments.belief_strength
                + w.charge_log_weight * self.experiments.charge_belief_strength
        };
        let max_log = pool
            .iter()
            .map(&log_weight)
            .fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<_> = pool
            .iter()
            .map(|w| (log_weight(w) - max_log).exp())
            .collect();
        if !weights.is_empty() {
            let sum: f64 = weights.iter().sum();
            self.last_belief_ess = sum * sum / weights.iter().map(|w| w * w).sum::<f64>();
        }
        let mut global_styles = [[0.0; 3]; 4];
        let mut runner_probabilities = [0.0; 4];
        let weight_sum: f64 = weights.iter().sum();
        for (index, world) in pool.iter().enumerate() {
            let weight = if charge_beliefs {
                weights[index] / weight_sum
            } else {
                1.0 / pool.len() as f64
            };
            for s in 0..4 {
                runner_probabilities[s] += weight * world.runners[s];
                for p in 0..3 {
                    // Retain baseline arithmetic/random streams when disabled.
                    global_styles[s][p] += if charge_beliefs {
                        weight * world.styles[s][p]
                    } else {
                        world.styles[s][p] / pool.len() as f64
                    };
                }
            }
        }
        let conservative_styles: [usize; 4] = std::array::from_fn(|seat| {
            let probabilities = global_styles[seat];
            let (style, confidence) = if self.experiments.collapse_equivalent_styles
                && !state.won.can_run(Seat::VALUES[seat])
            {
                (1, probabilities[1] + probabilities[2])
            } else {
                let style = if probabilities[1] > probabilities[2] {
                    1
                } else {
                    2
                };
                (style, probabilities[style])
            };
            if conservative
                && confidence >= self.experiments.style_confidence
                && pool
                    .iter()
                    .map(|w| self.history.decisions(w.hands, Seat::VALUES[seat]))
                    .sum::<usize>()
                    >= 3 * pool.len()
            {
                style
            } else {
                3
            }
        });
        let moon = self.config.moon && state.won.can_run(bot.seat) && !exact;
        let defense = (self.experiments.defensive_plan
            || self.experiments.disruptive_plan
            || self.experiments.learned_plan)
            && !exact;
        let modes: Vec<usize> = [0, 1, 2]
            .iter()
            .copied()
            .filter(|m| *m == 0 || (*m == 1 && moon) || (*m == 2 && defense))
            .collect();
        let mut totals = vec![[0.0; 3]; plays.len()];
        let mut half_differences = vec![[0.0; 2]; plays.len()];
        let mut counts = vec![0usize; plays.len()];
        let mut active: Vec<usize> = (0..plays.len()).collect();
        let budget = self.config.samples * plays.len();
        let mut spent = 0;
        let mut rounds = 0;
        let mut trees = SearchTree::default();
        let mut endgames = crate::endgame::EndgameSolver::new(self.experiments.canonical_endgames);
        let mut next_elimination = self.experiments.allocation_min.max(1);
        while spent + active.len() <= budget {
            if rounds > 0
                && self
                    .config
                    .time_limit
                    .map_or(false, |limit| start.elapsed() >= limit)
            {
                break;
            }
            let (hands, style_prob) = if charge_beliefs {
                (
                    pool[choose_weighted(&weights, &mut self.rng)].hands,
                    global_styles,
                )
            } else if conservative {
                (sampler.sample(&mut self.rng), global_styles)
            } else if infer {
                let world = &pool[choose_weighted(&weights, &mut self.rng)];
                (
                    world.hands,
                    if self.experiments.belief_strength > 0.0 {
                        world.styles
                    } else {
                        global_styles
                    },
                )
            } else {
                (sampler.sample(&mut self.rng), [[0.6, 0.2, 0.2]; 4])
            };
            let seed: u64 = self.rng.random();
            let mut styles = if conservative {
                Some(conservative_styles)
            } else if self.experiments.opponent_model {
                let mut rng = ChaCha12Rng::seed_from_u64(seed ^ 0x5354594c45);
                Some(style_prob.map(|w| choose_weighted(&w, &mut rng)))
            } else {
                None
            };
            if self.experiments.early_intent {
                let mut intent_rng = ChaCha12Rng::seed_from_u64(seed ^ 0x494e54454e54);
                let mut selected = styles.unwrap_or([3; 4]);
                for seat in Seat::VALUES {
                    if seat != bot.seat
                        // When adapting a human-policy fallback, scope the
                        // extra evidence to the same public human seats.
                        && (!self.experiments.human_model_fallback || self.human_seats[seat.idx()])
                        && selected[seat.idx()] == 3
                        && state.won.can_run(seat)
                        && runner_probabilities[seat.idx()] > 0.0
                        && intent_rng.random_bool(runner_probabilities[seat.idx()])
                    {
                        selected[seat.idx()] = 2;
                    }
                }
                styles = Some(selected);
            }
            if self.experiments.human_rollouts > 0.0 {
                let mut human_rng = ChaCha12Rng::seed_from_u64(seed ^ 0x48554d414e);
                let mut selected = styles.unwrap_or([3; 4]);
                for seat in Seat::VALUES {
                    if seat != bot.seat
                        && (!self.experiments.human_model_fallback || selected[seat.idx()] == 3)
                        && (!(self.experiments.skilled_model || self.experiments.action_model)
                            || self.human_seats[seat.idx()])
                        && human_rng.random_bool(self.experiments.human_rollouts)
                    {
                        selected[seat.idx()] = if self.experiments.action_model {
                            6
                        } else if self.experiments.skilled_model {
                            5
                        } else {
                            4
                        };
                    }
                }
                styles = Some(selected);
            }
            let mut defense_rng = ChaCha12Rng::seed_from_u64(seed ^ 0x444546454e44);
            let defenders = std::array::from_fn(|s| {
                self.experiments.moon_defense > 0.0
                    && !styles.map_or(false, |styles| styles[s] == 1 || styles[s] == 2)
                    && defense_rng.random_bool(self.experiments.moon_defense)
            });
            for &i in &active {
                for &mode in &modes {
                    let mut game = state.clone();
                    let event = GameEvent::Play {
                        seat: bot.seat,
                        card: plays[i],
                    };
                    let void = bot.void.on_event(state, &event);
                    game.apply(&event);
                    let won = if exact {
                        if self.experiments.cache_endgames {
                            endgames.solve(&game, &hands)
                        } else {
                            solve_endgame(&game, &hands)
                        }
                    } else {
                        trees.simulate(
                            &mut game,
                            void,
                            hands,
                            bot.seat,
                            mode == 1,
                            self.config.opponent_moon,
                            styles,
                            defenders,
                            self.experiments.dynamic_shooters,
                            mode == 2,
                            self.experiments.disruptive_plan,
                            self.experiments.learned_plan,
                            self.experiments.claim_rollouts,
                            self.experiments.early_shooters,
                            self.experiments.tree_depth,
                            if self.experiments.tree_depth > 0 {
                                vec![plays[i]]
                            } else {
                                Vec::new()
                            },
                            &mut ChaCha12Rng::seed_from_u64(seed),
                        );
                        game.won
                    };
                    let value = won.scores(state.charges).money(bot.seat) as f64;
                    totals[i][mode] += value;
                    if mode < 2 {
                        half_differences[i][rounds % 2] += if mode == 0 { -value } else { value };
                    }
                    self.last_evaluations += 1;
                }
                counts[i] += 1;
            }
            spent += active.len();
            rounds += 1;
            if self.experiments.allocate && !exact && active.len() > 2 && rounds == next_elimination
            {
                let mut values: Vec<_> = active
                    .iter()
                    .map(|&i| {
                        let mut mean = if moon
                            && (!self.experiments.moon_consensus
                                || agrees_to_shoot(half_differences[i], counts[i]))
                        {
                            totals[i][0].max(totals[i][1])
                        } else {
                            totals[i][0]
                        } / counts[i] as f64;
                        if defense {
                            mean = mean.max(totals[i][2] / counts[i] as f64);
                        }
                        let prior = self.policy.as_ref().map_or(0.0, |p| {
                            p.predict(&policy_features(bot, state, plays[i])) as f64
                        });
                        (i, mean + 4.0 / counts[i] as f64 * prior)
                    })
                    .collect();
                values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                active = values
                    .iter()
                    .take((active.len() + 1) / 2)
                    .map(|v| v.0)
                    .collect();
                next_elimination *= 2;
            }
        }
        self.last_tree_nodes = trees.nodes.len();
        (
            active
                .into_iter()
                .map(|i| {
                    (
                        plays[i],
                        (if moon
                            && (!self.experiments.moon_consensus
                                || agrees_to_shoot(half_differences[i], counts[i]))
                        {
                            totals[i][0].max(totals[i][1])
                        } else {
                            totals[i][0]
                        })
                        .max(if defense {
                            totals[i][2]
                        } else {
                            f64::NEG_INFINITY
                        }) as f32
                            / counts[i] as f32,
                    )
                })
                .collect(),
            rounds,
        )
    }
}

fn agrees_to_shoot(differences: [f64; 2], samples: usize) -> bool {
    samples >= 2 && differences.iter().all(|d| *d > 0.0)
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct NodeKey {
    own_plays: Vec<Card>,
    legal: u64,
    won: WonState,
    suit: u8,
    winning: bool,
    trick_len: u8,
    moon: bool,
    defense: bool,
}

#[derive(Default)]
struct SearchTree {
    nodes: HashMap<NodeKey, Vec<(Card, u32, f64)>>,
}

impl SearchTree {
    fn simulate<R: RngExt + ?Sized>(
        &mut self,
        state: &mut GameState,
        mut void: VoidState,
        hands: [Cards; 4],
        root: Seat,
        moon: bool,
        model_moon: bool,
        styles: Option<[usize; 4]>,
        defenders: [bool; 4],
        dynamic_shooters: bool,
        own_defense: bool,
        disruptive_plan: bool,
        learned_plan: bool,
        claim_rollouts: bool,
        early_shooters: bool,
        depth: usize,
        mut own_plays: Vec<Card>,
        rng: &mut R,
    ) {
        if depth == 0
            && styles.is_none()
            && !defenders.iter().any(|d| *d)
            && !dynamic_shooters
            && !own_defense
            && !claim_rollouts
            && !early_shooters
        {
            rollout(
                state,
                void,
                hands,
                root,
                if moon { Some(root) } else { None },
                model_moon,
                rng,
            );
            return;
        }
        let consider_shooting = (styles.is_none() || styles.unwrap().contains(&3))
            && model_moon
            && rng.random_bool(0.5);
        let heart_threshold = if early_shooters { 1 } else { 3 };
        let opponent_moon = if consider_shooting {
            Seat::VALUES.iter().copied().find(|s| {
                *s != root
                    && state.won.can_run(*s)
                    && (state.won.hearts(*s) >= heart_threshold || state.won.queen(*s))
            })
        } else {
            None
        };
        let mut path = Vec::new();
        let mut expanding = false;
        while state.played != Cards::ALL {
            if state.current_trick.is_empty() && state.played.contains_all(Cards::SCORING) {
                break;
            }
            let seat = state.next_actor.unwrap();
            let bot = BotState::with_void(seat, hands[seat.idx()], void);
            // Each actor sees only its own hand and public state. Only test at
            // the start of a trick: the public claim solver is cheap there.
            if claim_rollouts
                && state.current_trick.is_empty()
                && turbo_hearts_api::should_claim(state, void, seat, hands[seat.idx()])
            {
                state.won = state.won.win(seat, Cards::ALL - state.played);
                break;
            }
            let shooting = state.won.can_run(seat)
                && ((seat == root && moon)
                    || opponent_moon == Some(seat)
                    || (dynamic_shooters
                        && consider_shooting
                        && seat != root
                        && (state.won.hearts(seat) >= heart_threshold || state.won.queen(seat))));
            let defense = if seat == root && own_defense {
                if learned_plan {
                    Some(crate::HumanActionPolicy::play_with_rng(&bot, state, rng))
                } else if disruptive_plan {
                    crate::moon_defense::disruptive_play(&bot, state)
                } else {
                    crate::moon_defense::planned_play(&bot, state)
                }
            } else if defenders[seat.idx()] && !shooting {
                crate::moon_defense::defensive_play(&bot, state)
            } else {
                None
            };
            let card = if let Some(card) = defense {
                card
            } else if seat == root && path.len() < depth && !expanding {
                let legal = state
                    .legal_plays(bot.post_pass_hand)
                    .distinct_plays(state.played, state.current_trick);
                let key = NodeKey {
                    own_plays: own_plays.clone(),
                    legal: legal.bits,
                    won: state.won,
                    suit: if state.current_trick.is_empty() {
                        4
                    } else {
                        state.current_trick.suit().idx() as u8
                    },
                    winning: !state.current_trick.is_empty()
                        && state.current_trick.winning_seat(root) == root,
                    trick_len: state.current_trick.len() as u8,
                    moon,
                    defense: own_defense,
                };
                let preferred = if shooting {
                    GottaTryBot.choices(&bot, state)
                } else {
                    HeuristicBot.choices(&bot, state)
                };
                let actions = self.nodes.entry(key.clone()).or_insert_with(|| {
                    let mut cards: Vec<_> = legal.into_iter().collect();
                    cards.sort_by_key(|c| !preferred.contains(*c));
                    cards.into_iter().map(|c| (c, 0, 0.0)).collect()
                });
                let visits: u32 = actions.iter().map(|a| a.1).sum();
                let index = actions.iter().position(|a| a.1 == 0).unwrap_or_else(|| {
                    (0..actions.len())
                        .max_by(|&a, &b| {
                            let ucb = |i: usize| {
                                actions[i].2 / actions[i].1 as f64 / 100.0
                                    + 0.7 * ((visits as f64).ln() / actions[i].1 as f64).sqrt()
                            };
                            ucb(a).partial_cmp(&ucb(b)).unwrap()
                        })
                        .unwrap()
                });
                expanding = actions[index].1 == 0;
                let card = actions[index].0;
                path.push((key, index));
                own_plays.push(card);
                card
            } else if seat != root && styles.map_or(false, |s| s[seat.idx()] != 3) {
                style_play(styles.unwrap()[seat.idx()], &bot, state, rng)
            } else if shooting {
                GottaTryBot.play_with_rng(&bot, state, rng)
            } else {
                HeuristicBot.play_with_rng(&bot, state, rng)
            };
            let event = GameEvent::Play { seat, card };
            void = void.on_event(state, &event);
            state.apply(&event);
        }
        let money = state.scores().money(root) as f64;
        for (key, index) in path {
            let action = &mut self.nodes.get_mut(&key).unwrap()[index];
            action.1 += 1;
            action.2 += money;
        }
    }
}

pub(crate) fn style_play<R: RngExt + ?Sized>(
    style: usize,
    bot: &BotState,
    state: &GameState,
    rng: &mut R,
) -> Card {
    match style {
        1 => DuckBot.play_with_rng(bot, state, rng),
        2 => GottaTryBot.play_with_rng(bot, state, rng),
        4 => crate::HumanPolicy::play_with_rng(bot, state, rng),
        5 => crate::SkilledPolicy::play_with_rng(bot, state, rng),
        6 => crate::HumanActionPolicy::play_with_rng(bot, state, rng),
        _ => HeuristicBot.play_with_rng(bot, state, rng),
    }
}

pub(crate) fn simulate_style<R: RngExt + ?Sized>(
    state: &mut GameState,
    void: VoidState,
    hands: [Cards; 4],
    root: Seat,
    shoot: bool,
    styles: Option<[usize; 4]>,
    experiments: &SearchExperiments,
    rng: &mut R,
) {
    let mut styles = styles.map(|s| {
        s.map(|p| {
            if experiments.dynamic_shooters && p == 0 {
                3
            } else {
                p
            }
        })
    });
    if experiments.human_rollouts > 0.0 {
        let mut selected = styles.unwrap_or([3; 4]);
        for seat in Seat::VALUES {
            if seat != root
                && (!experiments.human_model_fallback || selected[seat.idx()] == 3)
                && rng.random_bool(experiments.human_rollouts)
            {
                selected[seat.idx()] = if experiments.action_model {
                    6
                } else if experiments.skilled_model {
                    5
                } else {
                    4
                };
            }
        }
        styles = Some(selected);
    }
    let defenders = std::array::from_fn(|s| {
        experiments.moon_defense > 0.0
            && !styles.map_or(false, |p| p[s] == 1 || p[s] == 2)
            && rng.random_bool(experiments.moon_defense)
    });
    SearchTree::default().simulate(
        state,
        void,
        hands,
        root,
        shoot,
        true,
        styles,
        defenders,
        experiments.dynamic_shooters,
        false,
        false,
        false,
        experiments.claim_rollouts,
        experiments.early_shooters,
        0,
        Vec::new(),
        rng,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use turbo_hearts_api::{GamePhase, PassDirection};

    #[test]
    fn safe_low_follow_is_soft_shooting_evidence_but_forced_or_dangerous_play_is_not() {
        use turbo_hearts_api::{ChargeState, Trick};
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.played = "2C 4S".parse().unwrap();
        state.current_trick = Trick::new().push(Card::FourSpades);
        state.charges = ChargeState::new().charge(Seat::North, Card::QueenSpades.into());
        let mut bot = BotState::new(Seat::East, "AK2S 2D".parse().unwrap());
        assert!(control_signal(&bot, &state, Card::TwoSpades).unwrap() > 0.0);
        assert!(control_signal(&bot, &state, Card::AceSpades).unwrap() < 0.0);
        let mut history = BehaviorHistory::default();
        history.on_event(
            &state,
            &GameEvent::Play {
                seat: Seat::East,
                card: Card::TwoSpades,
            },
        );
        let mut hands = [Cards::NONE; 4];
        hands[1] = bot.post_pass_hand;
        assert!(history.early_runner_probability(hands, Seat::East) > 0.1);
        assert_eq!(history.early_runner_probability(hands, Seat::West), 0.0);
        bot.post_pass_hand = "2S 2D".parse().unwrap();
        assert_eq!(control_signal(&bot, &state, Card::TwoSpades), None);
        bot.post_pass_hand = "AK2S 2D".parse().unwrap();
        state.charges = ChargeState::new();
        assert_eq!(control_signal(&bot, &state, Card::TwoSpades), None);
        state.current_trick = state.current_trick.push(Card::QueenSpades);
        assert_eq!(control_signal(&bot, &state, Card::TwoSpades), None);
    }

    #[test]
    fn claim_continuation_finishes_a_provable_moon_before_three_hearts() {
        // Actual remaining hands immediately after South's reported AH slough.
        let remaining: [Cards; 4] = ["T52H JT65D", "4S 73H Q983D", "J964H 742D", "QS KQ8H AKD 9C"]
            .map(|s| s.parse().unwrap());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.played = Cards::ALL - remaining.iter().fold(Cards::NONE, |a, h| a | *h);
        state.won = state.won.win(Seat::West, "AH TC".parse().unwrap());
        state.next_actor = Some(Seat::West);
        let mut rng = ChaCha12Rng::seed_from_u64(3);
        SearchTree::default().simulate(
            &mut state,
            VoidState::new(),
            remaining,
            Seat::South,
            false,
            true,
            Some([1; 4]),
            [false; 4],
            false,
            false,
            false,
            false,
            true,
            false,
            0,
            Vec::new(),
            &mut rng,
        );
        assert_eq!(state.won.hearts(Seat::West), 13);
        assert!(state.won.queen(Seat::West));
        assert_eq!(state.scores().score(Seat::West), -72);
    }

    #[test]
    fn recognizes_a_moon_that_emerges_after_the_rollout_starts() {
        let hands = [
            "Q652S 92H AD AKJT54C",
            "T83S T865H T85D 986C",
            "J94S AQ4H J32D Q732C",
            "AK7S KJ73H KQ9764D",
        ]
        .map(|s| s.parse().unwrap());
        let mut initial = GameState::new();
        initial.phase = GamePhase::PlayLeft;
        initial.next_actor = Some(Seat::South);
        assert_eq!(initial.won.hearts(Seat::West), 0);
        assert!(!initial.won.queen(Seat::West));
        let mut fixed = initial.clone();
        let mut dynamic = initial.clone();
        // rand 0.10's range sampler uses a different sequence than rand 0.8.
        // This seed retains the same four-heart vs. completed-moon regression.
        for (state, enabled) in [(&mut fixed, false), (&mut dynamic, true)] {
            SearchTree::default().simulate(
                state,
                VoidState::new(),
                hands,
                Seat::North,
                false,
                true,
                None,
                [false; 4],
                enabled,
                false,
                false,
                false,
                false,
                false,
                0,
                Vec::new(),
                &mut ChaCha12Rng::seed_from_u64(339),
            );
        }
        // West starts with no captured points. Once the opportunity emerges,
        // changing its continuation to shooting completes the run.
        assert_eq!(fixed.won.hearts(Seat::West), 4);
        assert_eq!(dynamic.won.hearts(Seat::West), 13);
        assert!(dynamic.won.queen(Seat::West));
    }

    #[test]
    fn posterior_is_normalized_stable_and_uses_the_prior_without_evidence() {
        let (prior, evidence) = posterior([0.0; 3]);
        for (a, b) in prior.iter().zip([0.6, 0.2, 0.2]) {
            assert!((a - b).abs() < 1e-12);
        }
        assert!(evidence.abs() < 1e-12);
        let (probabilities, evidence) = posterior([-10000.0, -10002.0, -9900.0]);
        assert!(evidence.is_finite());
        assert!((probabilities.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(probabilities[2] > 0.99);
    }

    #[test]
    fn choice_support_is_legal_and_likelihood_identifies_known_behaviors() {
        let mut rng = ChaCha12Rng::seed_from_u64(763);
        for style in 0..3 {
            let mut log_totals = [0.0; 3];
            for _ in 0..24 {
                let mut deck: Vec<_> = Cards::ALL.into_iter().collect();
                deck.shuffle(&mut rng);
                let hands: [Cards; 4] =
                    std::array::from_fn(|i| deck[13 * i..13 * (i + 1)].iter().copied().collect());
                let mut state = GameState::new();
                state.phase = GamePhase::PlayLeft;
                state.next_actor = Seat::VALUES
                    .iter()
                    .copied()
                    .find(|s| hands[s.idx()].contains(Card::TwoClubs));
                let mut history = BehaviorHistory::default();
                while state.played != Cards::ALL {
                    let seat = state.next_actor.unwrap();
                    let bot = BotState::with_void(seat, hands[seat.idx()], history.void);
                    let legal = state.legal_plays(bot.post_pass_hand);
                    let choices = [
                        HeuristicBot.choices(&bot, &state),
                        DuckBot.choices(&bot, &state),
                        GottaTryBot.choices(&bot, &state),
                    ];
                    for p in 0..3 {
                        assert!(!choices[p].is_empty());
                        assert!(legal.contains_all(choices[p]));
                        assert!(choices[p].contains(style_play(p, &bot, &state, &mut rng)));
                    }
                    let card = style_play(style, &bot, &state, &mut rng);
                    let event = GameEvent::Play { seat, card };
                    history.on_event(&state, &event);
                    state.apply(&event);
                }
                let logs = history.log_likelihoods(hands, Seat::North);
                assert_eq!(logs[0], [0.0; 3]);
                for other in 1..4 {
                    for p in 0..3 {
                        log_totals[p] += logs[other][p];
                    }
                }
                history.on_event(
                    &state,
                    &GameEvent::Deal {
                        north: Cards::NONE,
                        east: Cards::NONE,
                        south: Cards::NONE,
                        west: Cards::NONE,
                        pass: PassDirection::Left,
                    },
                );
                assert!(history.plays.is_empty());
                for seat in Seat::VALUES {
                    for suit in turbo_hearts_api::Suit::VALUES {
                        assert!(!history.void.is_void(seat, suit));
                    }
                }
            }
            for other in 0..3 {
                if other != style {
                    assert!(
                        log_totals[style] > log_totals[other],
                        "style {}: {:?}",
                        style,
                        log_totals
                    );
                }
            }
        }
    }
}
