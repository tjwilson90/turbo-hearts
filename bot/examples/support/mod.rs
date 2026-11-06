#![allow(dead_code)]
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::time::Instant;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

pub struct Player {
    pub search: Option<NeuralNetworkBot>,
    kind: String,
    pub network: Option<PolicyNetwork>,
    temperature: f64,
    rng: ChaCha12Rng,
}

impl Player {
    pub fn new(kind: &str, seed: u64) -> Self {
        let parts: Vec<_> = kind.split(':').collect();
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
        let search = if network.is_some() {
            None
        } else if kind == "default" {
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
                ["heuristic", "duck", "random", "gottatry"].contains(&kind),
                "unknown player: {}",
                kind
            );
            None
        };
        let search = search.map(|search| {
            if kind == "default" {
                return search.with_experiments(SearchExperiments::production());
            }
            search.with_experiments(SearchExperiments {
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
        if let Some(search) = &mut self.search {
            search.on_event(bot, state, event);
        }
    }
}

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
        let player = turbo_hearts_api::Player::Human {
            user_id: UserId::null(),
        };
        result.apply(GameEvent::Sit {
            north: player.clone(),
            east: player.clone(),
            south: player.clone(),
            west: player,
            rules,
            seed: Seed::Chosen {
                value: format!("arena-{}", seed),
            },
        });
        result.apply(result.game.seed.deal(direction));
        while !result.game.state.phase.is_playing() {
            if result.game.state.phase.is_passing() {
                result.pass();
            } else {
                result.charge();
            }
        }
        result
    }

    fn apply(&mut self, event: GameEvent) {
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
        let phase = self.game.state.phase;
        let passes: [Cards; 4] = std::array::from_fn(|i| {
            let start = Instant::now();
            let cards = self.players[i].pass(&self.states[i], &self.observations[i]);
            self.preplay_micros[i] += start.elapsed().as_micros();
            cards
        });
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
        let mut turns = 0;
        while self.game.state.phase.is_charging() {
            for i in 0..4 {
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
        }
    }

    pub fn play_one(&mut self) {
        let seat = self.game.state.next_actor.unwrap();
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
