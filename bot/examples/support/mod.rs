#![allow(dead_code)]
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

#[derive(Clone)]
pub struct Player {
    pub search: Option<NeuralNetworkBot>,
    kind: String,
    pub network: Option<PolicyNetwork>,
    temperature: f64,
    human_network: Option<Arc<HumanNetwork>>,
    memory: HumanMemory,
    rng: ChaCha12Rng,
}

impl Player {
    pub fn new(kind: &str, seed: u64) -> Self {
        let parts: Vec<_> = kind.split(':').collect();
        if parts[0] == "prelearn" {
            assert!(parts.len() >= 3, "prelearn:PASS_MODEL:CHARGE_MODEL[:SAMPLES:EXACT:TIME_MS:MARGIN:PASS_MASK:HUMAN_RECIPIENTS]; none disables a model");
            let mut base = format!(
                "round7:{}:{}",
                parts.get(3).unwrap_or(&"128"),
                parts.get(4).unwrap_or(&"12")
            );
            if let Some(time) = parts.get(5) {
                base.push(':');
                base.push_str(time);
            }
            let mut player = Self::new(&base, seed);
            let load = |path: &str| {
                if path == "none" {
                    None
                } else {
                    Some(PreplayNetwork::load(path).expect("preplay checkpoint"))
                }
            };
            player.search = player.search.map(|s| {
                let pass_mask: u8 = parts.get(7).map_or(15, |v| v.parse().unwrap());
                assert!(pass_mask < 16);
                s.with_preplay_models(
                    load(parts[1]),
                    load(parts[2]),
                    parts.get(6).map_or(0.0, |v| v.parse().unwrap()),
                )
                .with_preplay_pass_directions(std::array::from_fn(|i| pass_mask & (1 << i) != 0))
                .with_preplay_human_recipients(
                    match parts.get(8).copied().unwrap_or("0") {
                        "0" => false,
                        "1" => true,
                        _ => panic!("human recipient flag must be 0 or 1"),
                    },
                )
            });
            player.kind = kind.to_owned();
            return player;
        }

        let network = if parts[0] == "policy" {
            Some(PolicyNetwork::load(parts[1]).expect("policy checkpoint"))
        } else {
            None
        };
        let temperature = if network.is_some() {
            parts.get(2).unwrap_or(&"0").parse().expect("temperature")
        } else {
            0.0
        };
        let human_network = if ["neuralplayer", "neuralhybrid"].contains(&parts[0]) {
            Some(Arc::new(
                HumanNetwork::load(parts[1]).expect("human policy checkpoint"),
            ))
        } else {
            None
        };
        let search = if parts[0] == "neuralplayer" {
            None
        } else if [
            "round7",
            "coherent",
            "neural",
            "neuralmix",
            "payoff",
            "neuralhybrid",
        ]
        .contains(&parts[0])
        {
            let base = if ["round7", "coherent"].contains(&parts[0]) {
                1
            } else {
                2
            };
            Some(NeuralNetworkBot::with_config(
                SearchConfig {
                    samples: parts.get(base).map_or(128, |v| v.parse().unwrap()),
                    exact_cards: parts.get(base + 1).map_or(12, |v| v.parse().unwrap()),
                    time_limit: parts
                        .get(base + 2)
                        .filter(|v| **v != "none")
                        .map(|v| Duration::from_millis(v.parse().unwrap())),
                    ..SearchConfig::default()
                },
                seed,
            ))
        } else if network.is_some() {
            None
        } else if ["default", "round4", "skilledhybrid", "actionhybrid"].contains(&kind) {
            Some(NeuralNetworkBot::with_config(
                SearchConfig {
                    time_limit: None,
                    ..SearchConfig::default()
                },
                seed,
            ))
        } else if [
            "search",
            "moon",
            "defend",
            "model",
            "belief",
            "posterior",
            "tree",
            "modeltree",
            "allocate",
            "guided",
            "preplay",
            "pass",
            "charge",
            "combined",
            "modelcharge",
            "posteriorcharge",
            "guard",
            "cautious",
            "cautiousguard",
            "consensus",
            "guardconsensus",
            "dynamic",
            "dynamicguard",
            "cached",
            "selective",
            "canonical",
            "cautiousdynamic",
            "dynamiccharge",
            "plan",
            "claims",
            "early",
            "preserve",
            "humanlike",
            "planclaims",
            "earlyclaims",
            "humanmodel",
            "humanmix",
            "safemix",
            "safeclaims",
            "safeearly",
            "safe",
            "chargebelief",
            "intent",
            "disrupt",
            "humanaware",
            "skilledmix",
            "skilledmodel",
            "skilledintent",
            "skilledclaims",
            "skilledfallback",
            "skilledadaptive",
            "actionmix",
            "actionmodel",
            "actionintent",
            "actionclaims",
            "actionplan",
        ]
        .contains(&parts[0])
        {
            Some(NeuralNetworkBot::with_config(
                SearchConfig {
                    moon: parts[0] != "search",
                    opponent_moon: !["search", "moon"].contains(&parts[0]),
                    samples: parts
                        .get(1)
                        .unwrap_or(&"128")
                        .parse()
                        .expect("sample count"),
                    exact_cards: parts.get(2).unwrap_or(&"12").parse().expect("exact cards"),
                    time_limit: None,
                },
                seed,
            ))
        } else {
            assert!(
                [
                    "heuristic",
                    "duck",
                    "random",
                    "gottatry",
                    "human",
                    "humanclaim",
                    "skilled",
                    "skilledclaim",
                    "action",
                    "actionclaim"
                ]
                .contains(&kind),
                "unknown player: {}",
                kind
            );
            None
        };
        let search = search.map(|search| {
            if ["round7", "neuralhybrid"].contains(&parts[0]) {
                return search.with_experiments(SearchExperiments::round_seven());
            }
            if parts[0] == "coherent" {
                return search.with_experiments(SearchExperiments {
                    coherent_humans: true,
                    ..SearchExperiments::round_seven()
                });
            }
            if ["neural", "neuralmix"].contains(&parts[0]) {
                return search
                    .with_experiments(SearchExperiments {
                        human_rollouts: if parts[0] == "neuralmix" { 0.5 } else { 1.0 },
                        ..SearchExperiments::round_seven()
                    })
                    .with_human_network(
                        HumanNetwork::load(parts[1]).expect("human policy checkpoint"),
                    );
            }
            if parts[0] == "payoff" {
                return search
                    .with_experiments(SearchExperiments::round_seven())
                    .with_payoff_network(
                        HumanNetwork::load(parts[1]).expect("payoff checkpoint"),
                        parts.get(5).map_or(0.25, |v| v.parse().unwrap()),
                    );
            }

            if kind == "default" {
                return search
                    .with_experiments(SearchExperiments::production())
                    .with_production_preplay();
            }
            if ["round4", "skilledhybrid", "actionhybrid"].contains(&kind) {
                return search.with_experiments(SearchExperiments::round_four());
            }
            if [
                "actionmix",
                "actionmodel",
                "actionintent",
                "actionclaims",
                "actionplan",
            ]
            .contains(&parts[0])
            {
                return search.with_experiments(SearchExperiments {
                    action_model: true,
                    human_rollouts: if parts[0] == "actionmodel" { 1.0 } else { 0.5 },
                    early_intent: parts[0] == "actionintent",
                    claim_rollouts: parts[0] == "actionclaims",
                    learned_plan: parts[0] == "actionplan",
                    ..SearchExperiments::round_four()
                });
            }
            if [
                "skilledmix",
                "skilledmodel",
                "skilledintent",
                "skilledclaims",
                "skilledfallback",
                "skilledadaptive",
            ]
            .contains(&parts[0])
            {
                return search.with_experiments(SearchExperiments {
                    skilled_model: true,
                    human_rollouts: if ["skilledmodel", "skilledfallback", "skilledadaptive"]
                        .contains(&parts[0])
                    {
                        1.0
                    } else {
                        0.5
                    },
                    human_model_fallback: ["skilledfallback", "skilledadaptive"]
                        .contains(&parts[0]),
                    early_intent: ["skilledintent", "skilledadaptive"].contains(&parts[0]),
                    claim_rollouts: parts[0] == "skilledclaims",
                    ..SearchExperiments::round_four()
                });
            }
            if ["chargebelief", "intent", "disrupt", "humanaware"].contains(&parts[0]) {
                return search.with_experiments(SearchExperiments {
                    charge_belief_strength: if ["chargebelief", "humanaware"].contains(&parts[0]) {
                        parts.get(6).map_or(0.5, |s| s.parse().unwrap())
                    } else {
                        0.0
                    },
                    early_intent: ["intent", "humanaware"].contains(&parts[0]),
                    disruptive_plan: ["disrupt", "humanaware"].contains(&parts[0]),
                    ..SearchExperiments::round_four()
                });
            }
            if [
                "plan",
                "claims",
                "early",
                "preserve",
                "humanlike",
                "planclaims",
                "earlyclaims",
                "humanmodel",
                "humanmix",
                "safemix",
                "safeclaims",
                "safeearly",
                "safe",
            ]
            .contains(&parts[0])
            {
                return search.with_experiments(SearchExperiments {
                    defensive_plan: ["plan", "humanlike", "planclaims"].contains(&parts[0]),
                    claim_rollouts: [
                        "claims",
                        "humanlike",
                        "planclaims",
                        "earlyclaims",
                        "safeclaims",
                        "safeearly",
                        "safemix",
                    ]
                    .contains(&parts[0]),
                    early_shooters: ["early", "humanlike", "earlyclaims", "safeearly", "safe"]
                        .contains(&parts[0]),
                    stopper_margin: if ["preserve", "safeclaims", "safeearly", "safemix", "safe"]
                        .contains(&parts[0])
                    {
                        parts.get(6).map_or(4.0, |s| s.parse().unwrap())
                    } else {
                        0.0
                    },
                    human_rollouts: match parts[0] {
                        "humanmodel" => 1.0,
                        "humanmix" | "safemix" => 0.5,
                        _ => 0.0,
                    },
                    ..SearchExperiments::round_three()
                });
            }
            search.with_experiments(SearchExperiments {
                learned_plan: false,
                coherent_humans: false,
                human_model_fallback: false,
                action_model: false,
                skilled_model: false,
                charge_belief_strength: 0.0,
                early_intent: false,
                disruptive_plan: false,
                defensive_plan: false,
                claim_rollouts: false,
                early_shooters: false,
                stopper_margin: 0.0,
                human_rollouts: 0.0,
                conservative_model: ["cautious", "cautiousguard", "selective", "cautiousdynamic"]
                    .contains(&parts[0]),
                style_confidence: parts
                    .get(7)
                    .map(|s| s.parse().expect("style confidence"))
                    .unwrap_or(0.9),
                collapse_equivalent_styles: parts[0] == "selective",
                moon_defense: if ["guard", "cautiousguard", "guardconsensus", "dynamicguard"]
                    .contains(&parts[0])
                {
                    parts
                        .get(6)
                        .map(|s| s.parse().expect("defense probability"))
                        .unwrap_or(1.0)
                } else {
                    0.0
                },
                moon_consensus: ["consensus", "guardconsensus"].contains(&parts[0]),
                dynamic_shooters: [
                    "dynamic",
                    "dynamicguard",
                    "cautiousdynamic",
                    "dynamiccharge",
                ]
                .contains(&parts[0]),
                cache_endgames: ["cached", "canonical"].contains(&parts[0]),
                canonical_endgames: parts[0] == "canonical",
                opponent_model: [
                    "model",
                    "posterior",
                    "combined",
                    "modeltree",
                    "modelcharge",
                    "posteriorcharge",
                ]
                .contains(&parts[0]),
                belief_strength: if [
                    "belief",
                    "posterior",
                    "combined",
                    "modeltree",
                    "posteriorcharge",
                ]
                .contains(&parts[0])
                {
                    0.5
                } else {
                    0.0
                },
                tree_depth: if ["tree", "modeltree"].contains(&parts[0]) {
                    2
                } else {
                    0
                },
                allocate: ["allocate", "guided"].contains(&parts[0]),
                allocation_min: parts
                    .get(5)
                    .map(|s| s.parse().expect("allocation minimum"))
                    .unwrap_or(32),
                pass_samples: if ["preplay", "pass", "combined"].contains(&parts[0]) {
                    32
                } else {
                    0
                },
                charge_samples: if [
                    "preplay",
                    "charge",
                    "combined",
                    "modelcharge",
                    "posteriorcharge",
                    "dynamiccharge",
                ]
                .contains(&parts[0])
                {
                    32
                } else {
                    0
                },
            })
        });
        let search = search.map(|search| {
            if [
                "round7",
                "coherent",
                "neural",
                "neuralmix",
                "payoff",
                "neuralhybrid",
            ]
            .contains(&parts[0])
            {
                return search;
            }
            if let Some(path) = parts.get(3) {
                if *path == "none" {
                    search
                } else {
                    search.with_policy(
                        PolicyNetwork::load(path).expect("policy model"),
                        parts
                            .get(4)
                            .unwrap_or(&"16")
                            .parse()
                            .expect("prior samples"),
                    )
                }
            } else {
                search
            }
        });
        Self {
            search,
            human_network,
            memory: HumanMemory::default(),
            kind: kind.to_owned(),
            network,
            temperature,
            rng: ChaCha12Rng::seed_from_u64(seed),
        }
    }
    pub fn policy_distribution(&self, bot: &BotState, state: &GameState) -> Vec<(Card, f64)> {
        let network = self.network.as_ref().expect("policy player");
        let plays = state
            .legal_plays(bot.post_pass_hand)
            .distinct_plays(state.played, state.current_trick);
        let values: Vec<_> = plays
            .into_iter()
            .map(|c| (c, network.predict(&policy_features(bot, state, c)) as f64))
            .collect();
        let best = values.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
        if self.temperature == 0.0 {
            let choice = values.iter().position(|v| v.1 == best).unwrap();
            return values
                .iter()
                .enumerate()
                .map(|(i, v)| (v.0, (i == choice) as u8 as f64))
                .collect();
        }
        assert!(self.temperature > 0.0 && self.temperature.is_finite());
        let weights: Vec<_> = values
            .iter()
            .map(|v| ((v.1 - best) / self.temperature).max(-60.0).exp())
            .collect();
        let sum: f64 = weights.iter().sum();
        values
            .iter()
            .zip(weights)
            .map(|(v, w)| (v.0, w / sum))
            .collect()
    }
}

impl Algorithm for Player {
    fn pass(&mut self, bot: &BotState, state: &GameState) -> Cards {
        if let Some(search) = &mut self.search {
            return search.pass(bot, state);
        }
        if self.kind == "gottatry" {
            return GottaTryBot.pass(bot, state);
        }
        if self.kind == "duck" {
            DuckBot.pass(bot, state)
        } else {
            HeuristicBot.pass(bot, state)
        }
    }
    fn charge(&mut self, bot: &BotState, state: &GameState) -> Cards {
        if let Some(search) = &mut self.search {
            return search.charge(bot, state);
        }
        if self.kind == "gottatry" {
            return GottaTryBot.charge(bot, state);
        }
        if self.kind == "duck" {
            Cards::NONE
        } else {
            HeuristicBot.charge_with_rng(bot, state, &mut self.rng)
        }
    }
    fn play(&mut self, bot: &BotState, state: &GameState) -> Card {
        if let Some(network) = &self.human_network {
            if self.search.is_none() || self.rng.random_bool(0.5) {
                return network.play(bot, state, &self.memory, &mut self.rng);
            }
        }
        if self.kind == "action"
            || self.kind == "actionclaim"
            || (self.kind == "actionhybrid" && self.rng.random_bool(0.5))
        {
            return HumanActionPolicy::play_with_rng(bot, state, &mut self.rng);
        }
        if self.kind == "skilled"
            || self.kind == "skilledclaim"
            || (self.kind == "skilledhybrid" && self.rng.random_bool(0.5))
        {
            return SkilledPolicy::play_with_rng(bot, state, &mut self.rng);
        }
        if self.network.is_some() {
            let distribution = self.policy_distribution(bot, state);
            let mut target = self.rng.random::<f64>();
            for (card, p) in &distribution {
                target -= p;
                if target <= 0.0 {
                    return *card;
                }
            }
            return distribution.last().unwrap().0;
        }
        if let Some(search) = &mut self.search {
            return search.play(bot, state);
        }
        if self.kind == "human" || self.kind == "humanclaim" {
            return HumanPolicy::play_with_rng(bot, state, &mut self.rng);
        }
        if self.kind == "gottatry" {
            return GottaTryBot.play_with_rng(bot, state, &mut self.rng);
        }
        if self.kind == "random" || (self.kind == "duck" && state.current_trick.is_empty()) {
            let legal = state.legal_plays(bot.post_pass_hand);
            return legal
                .into_iter()
                .nth(self.rng.random_range(0..legal.len()))
                .unwrap();
        }
        if self.kind == "duck" {
            DuckBot.play(bot, state)
        } else {
            HeuristicBot.play_with_rng(bot, state, &mut self.rng)
        }
    }
    fn on_event(&mut self, bot: &BotState, state: &GameState, event: &GameEvent) {
        self.memory.on_event(bot, state, event);
        if let Some(search) = &mut self.search {
            search.on_event(bot, state, event);
        }
    }
}

#[derive(Clone)]
pub struct Match {
    pub game: Game<()>,
    pub states: [BotState; 4],
    /// The same redacted public state each server-side bot receives.
    pub observations: [GameState; 4],
    pub players: [Player; 4],
    pub play_micros: [u128; 4],
    pub preplay_micros: [u128; 4],
    pub root_evaluations: [usize; 4],
    pub max_play_micros: [u128; 4],
}

impl Match {
    pub fn new(
        seed: u64,
        direction: PassDirection,
        kinds: [&str; 4],
        rules: ChargingRules,
    ) -> Self {
        let mut result = Self::dealt(seed, direction, kinds, rules);
        result.finish_preplay();
        result
    }

    pub fn dealt(
        seed: u64,
        direction: PassDirection,
        kinds: [&str; 4],
        rules: ChargingRules,
    ) -> Self {
        Self::dealt_with_humans(seed, direction, kinds, rules, [true; 4])
    }

    pub fn dealt_with_humans(
        seed: u64,
        direction: PassDirection,
        kinds: [&str; 4],
        rules: ChargingRules,
        humans: [bool; 4],
    ) -> Self {
        let mut game = Game::new();
        game.seed = HashedSeed::from(&Seed::Chosen {
            value: format!("arena-{}", seed),
        });
        game.state.phase = match direction {
            PassDirection::Left => GamePhase::PassLeft,
            PassDirection::Right => GamePhase::PassRight,
            PassDirection::Across => GamePhase::PassAcross,
            PassDirection::Keeper => GamePhase::ChargeKeeper1,
        };
        game.state.rules = rules;
        let mut result = Self {
            observations: std::array::from_fn(|_| game.state.clone()),
            game,
            states: Seat::VALUES.map(|s| BotState::new(s, Cards::NONE)),
            players: std::array::from_fn(|i| {
                Player::new(kinds[i], seed.wrapping_mul(4).wrapping_add(i as u64))
            }),
            play_micros: [0; 4],
            preplay_micros: [0; 4],
            root_evaluations: [0; 4],
            max_play_micros: [0; 4],
        };
        let players: [_; 4] = std::array::from_fn(|i| {
            if humans[i] {
                turbo_hearts_api::Player::Human {
                    user_id: UserId::null(),
                }
            } else {
                turbo_hearts_api::Player::Bot {
                    user_id: UserId::null(),
                    strategy: BotStrategy::NeuralNet,
                }
            }
        });
        result.apply(GameEvent::Sit {
            north: players[0].clone(),
            east: players[1].clone(),
            south: players[2].clone(),
            west: players[3].clone(),
            rules,
            seed: Seed::Chosen {
                value: format!("arena-{}", seed),
            },
        });
        result.apply(result.game.seed.deal(direction));
        result
    }

    pub fn finish_preplay(&mut self) {
        while !self.game.state.phase.is_playing() {
            if self.game.state.phase.is_passing() {
                self.pass();
            } else {
                self.charge();
            }
        }
    }

    pub fn reseed_players(&mut self, seed: u64) {
        for (i, player) in self.players.iter_mut().enumerate() {
            player.rng = ChaCha12Rng::seed_from_u64(seed * 4 + i as u64);
            if let Some(search) = &mut player.search {
                search.reseed(seed * 4 + i as u64);
            }
        }
    }

    pub fn apply(&mut self, event: GameEvent) {
        let Self {
            game,
            states,
            players,
            observations,
            ..
        } = self;
        game.apply(&event, |game, event| {
            for i in 0..4 {
                let event = event.redact(Some(Seat::VALUES[i]), game.state.rules);
                let bot = &mut states[i];
                match &event {
                    GameEvent::Deal {
                        north,
                        east,
                        south,
                        west,
                        ..
                    } => {
                        bot.pre_pass_hand = [*north, *east, *south, *west][i];
                        bot.post_pass_hand = bot.pre_pass_hand;
                    }
                    GameEvent::SendPass { from, cards } if from.idx() == i => {
                        bot.post_pass_hand -= *cards
                    }
                    GameEvent::RecvPass { to, cards } if to.idx() == i => {
                        bot.post_pass_hand |= *cards
                    }
                    _ => {}
                }
                bot.on_event(&observations[i], &event);
                players[i].on_event(bot, &observations[i], &event);
                observations[i].apply(&event);
            }
        });
    }

    fn pass(&mut self) {
        let passes: [Cards; 4] = std::array::from_fn(|i| {
            let start = Instant::now();
            let cards = self.players[i].pass(&self.states[i], &self.observations[i]);
            self.preplay_micros[i] += start.elapsed().as_micros();
            cards
        });
        self.apply_passes(passes);
    }

    pub fn apply_passes(&mut self, passes: [Cards; 4]) {
        let phase = self.game.state.phase;
        for i in 0..4 {
            assert!(self.states[i].pre_pass_hand.contains_all(passes[i]));
            assert_eq!(passes[i].len(), 3);
            self.apply(GameEvent::SendPass {
                from: Seat::VALUES[i],
                cards: passes[i],
            });
        }
        if phase == GamePhase::PassKeeper {
            for event in self.game.seed.keeper_pass(self.game.post_pass_hand) {
                self.apply(event);
            }
        } else {
            for i in 0..4 {
                self.apply(GameEvent::RecvPass {
                    to: phase.pass_receiver(Seat::VALUES[i]),
                    cards: passes[i],
                });
            }
        }
    }

    fn charge(&mut self) {
        self.charge_from(0);
    }

    pub fn charge_from(&mut self, mut first: usize) {
        let mut turns = 0;
        while self.game.state.phase.is_charging() {
            for i in first..4 {
                if !self.game.state.phase.is_charging() {
                    break;
                }
                let s = Seat::VALUES[i];
                if self.game.state.can_charge(s) && !self.game.state.done.charged(s) {
                    let start = Instant::now();
                    let cards = self.players[i].charge(&self.states[i], &self.observations[i]);
                    self.preplay_micros[i] += start.elapsed().as_micros();
                    assert!(self.states[i].post_pass_hand.contains_all(cards));
                    assert!(Cards::CHARGEABLE.contains_all(cards));
                    assert!(!self.game.state.charges.all_charges().contains_any(cards));
                    self.apply(GameEvent::Charge { seat: s, cards });
                    turns += 1;
                    assert!(turns <= 32, "charging did not terminate");
                }
            }
            first = 0;
        }
    }

    pub fn play_one(&mut self) {
        let seat = self.game.state.next_actor.unwrap();
        if [
            "humanclaim",
            "skilledclaim",
            "skilledhybrid",
            "actionclaim",
            "actionhybrid",
            "neuralplayer",
            "neuralhybrid",
        ]
        .contains(&self.players[seat.idx()].kind.split(':').next().unwrap())
            && should_claim(
                &self.observations[seat.idx()],
                self.states[seat.idx()].void,
                seat,
                self.states[seat.idx()].post_pass_hand,
            )
        {
            self.apply(GameEvent::Claim {
                seat,
                hand: self.states[seat.idx()].post_pass_hand - self.game.state.played,
            });
            for acceptor in Seat::VALUES {
                if acceptor != seat {
                    self.apply(GameEvent::AcceptClaim {
                        claimer: seat,
                        acceptor,
                    });
                }
            }
            return;
        }
        let start = Instant::now();
        let card =
            self.players[seat.idx()].play(&self.states[seat.idx()], &self.observations[seat.idx()]);
        let micros = start.elapsed().as_micros();
        self.root_evaluations[seat.idx()] += self.players[seat.idx()]
            .search
            .as_ref()
            .map_or(0, |b| b.last_search_evaluations());
        self.play_micros[seat.idx()] += micros;
        self.max_play_micros[seat.idx()] = self.max_play_micros[seat.idx()].max(micros);
        self.play_card(card);
    }

    pub fn play_card(&mut self, card: Card) {
        let seat = self.game.state.next_actor.unwrap();
        assert!(
            self.game
                .state
                .legal_plays(self.states[seat.idx()].post_pass_hand)
                .contains(card),
            "illegal play"
        );
        self.apply(GameEvent::Play { seat, card });
    }

    pub fn finish(&mut self) -> Scores {
        while self.game.state.phase.is_playing() {
            self.play_one();
        }
        self.game.state.scores()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blind_charges_stay_private_until_revealed() {
        let mut game = Match::new(61, PassDirection::Left, ["duck"; 4], ChargingRules::Blind);
        // Start another charging phase using the same ownership. All duck
        // players passed in the first phase, so no old charges can interfere.
        game.game.state.phase = GamePhase::ChargeLeft;
        game.game.state.next_actor = None;
        for state in &mut game.observations {
            state.phase = GamePhase::ChargeLeft;
            state.next_actor = None;
        }
        let owner = Seat::VALUES
            .iter()
            .copied()
            .find(|s| {
                game.states[s.idx()]
                    .post_pass_hand
                    .contains(Card::AceHearts)
            })
            .unwrap();
        game.apply(GameEvent::Charge {
            seat: owner,
            cards: Card::AceHearts.into(),
        });
        assert!(game.game.state.charges.is_charged(Card::AceHearts));
        for seat in Seat::VALUES {
            assert_eq!(
                game.observations[seat.idx()]
                    .charges
                    .is_charged(Card::AceHearts),
                seat == owner
            );
            assert_eq!(game.observations[seat.idx()].charge_count, 1);
        }
        game.charge();
        for state in &game.observations {
            assert!(state.charges.is_charged(Card::AceHearts));
        }
    }
}
