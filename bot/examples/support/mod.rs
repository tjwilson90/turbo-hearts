#![allow(dead_code)]
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;
use turbo_hearts_api::*;
use turbo_hearts_bot::*;

pub struct Player {
    pub search: Option<NeuralNetworkBot>,
    kind: String,
    rng: StdRng,
}

impl Player {
    pub fn new(kind: &str, seed: u64) -> Self {
        let parts: Vec<_> = kind.split(':').collect();
        let search = if kind == "default" {
            Some(NeuralNetworkBot::with_config(
                SearchConfig {
                    time_limit: None,
                    ..SearchConfig::default()
                },
                seed,
            ))
        } else if ["search", "moon", "defend"].contains(&parts[0]) {
            Some(NeuralNetworkBot::with_config(
                SearchConfig {
                    moon: parts[0] != "search",
                    opponent_moon: parts[0] == "defend",
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
            rng: StdRng::seed_from_u64(seed),
        }
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
                .nth(self.rng.gen_range(0..legal.len()))
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
    pub players: [Player; 4],
    pub play_micros: [u128; 4],
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
            game,
            states: Seat::VALUES.map(|s| BotState::new(s, Cards::NONE)),
            players: std::array::from_fn(|i| {
                Player::new(kinds[i], seed.wrapping_mul(4).wrapping_add(i as u64))
            }),
            play_micros: [0; 4],
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
                bot.on_event(&game.state, &event);
                players[i].on_event(bot, &game.state, &event);
            }
        });
    }

    fn pass(&mut self) {
        let phase = self.game.state.phase;
        let passes: [Cards; 4] =
            std::array::from_fn(|i| self.players[i].pass(&self.states[i], &self.game.state));
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
                    let cards = self.players[i].charge(&self.states[i], &self.game.state);
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
        let card = self.players[seat.idx()].play(&self.states[seat.idx()], &self.game.state);
        let micros = start.elapsed().as_micros();
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
