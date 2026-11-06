use crate::{policy_features, Algorithm, GottaTryBot, HandMaker, HeuristicBot, PolicyNetwork};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::{Duration, Instant};
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState, Seat, VoidState, WonState};

/// Search settings. Fixed samples and a seed make experiments reproducible.
#[derive(Clone, Debug)]
pub struct SearchConfig {
    /// Maximum shared hidden deals per decision (must be nonzero).
    pub samples: usize,
    /// Compare point avoidance with a dedicated moon continuation.
    pub moon: bool,
    /// Mix in a shooting policy for opponents with an unbroken moon threat.
    pub opponent_moon: bool,
    /// Use full max-n search when at most this many cards remain (0 disables).
    pub exact_cards: usize,
    /// Soft budget: finish at least one complete batch of all candidates.
    /// None gives reproducible fixed-sample evaluation.
    pub time_limit: Option<Duration>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            samples: 128,
            moon: true,
            opponent_moon: true,
            exact_cards: 12,
            time_limit: Some(Duration::from_millis(1000)),
        }
    }
}

/// The historical `neural_net` strategy, now able to play without external weights.
#[derive(Clone)]
pub struct NeuralNetworkBot {
    hand_maker: HandMaker,
    rng: StdRng,
    charge_rng: StdRng,
    config: SearchConfig,
    policy: Option<PolicyNetwork>,
    prior_samples: f32,
}

impl NeuralNetworkBot {
    pub fn new() -> Self {
        Self::with_config(SearchConfig::default(), rand::thread_rng().gen())
    }

    pub fn with_config(config: SearchConfig, seed: u64) -> Self {
        assert!(config.samples > 0, "search needs at least one sample");
        assert!(
            config.exact_cards <= 16,
            "exact search is limited to small endgames"
        );
        Self {
            hand_maker: HandMaker::new(),
            rng: StdRng::seed_from_u64(seed),
            charge_rng: StdRng::seed_from_u64(seed),
            config,
            policy: None,
            prior_samples: 0.0,
        }
    }

    /// Experimental learned advantage prior, in units of additional samples.
    /// Disabled by default: offline prediction accuracy alone is not evidence
    /// that blending it into search improves actual games.
    pub fn with_policy(mut self, policy: PolicyNetwork, prior_samples: f32) -> Self {
        assert!(prior_samples.is_finite() && prior_samples >= 0.0);
        self.policy = Some(policy);
        self.prior_samples = prior_samples;
        self
    }

    /// Expected money for each distinct legal move. All moves see the same deals
    /// and random rollout streams, reducing variance in their differences.
    pub fn evaluate(&mut self, bot: &BotState, state: &GameState) -> Vec<(Card, f32)> {
        self.search(bot, state).0
    }

    fn search(&mut self, bot: &BotState, state: &GameState) -> (Vec<(Card, f32)>, usize) {
        let plays: Vec<_> = state
            .legal_plays(bot.post_pass_hand)
            .distinct_plays(state.played, state.current_trick)
            .into_iter()
            .collect();
        let mut totals = vec![0.0; plays.len()];
        let mut moon_totals = vec![0.0; plays.len()];
        let try_moon = self.config.moon
            && state.won.can_run(bot.seat)
            && 52 - state.played.len() > self.config.exact_cards;
        let start = Instant::now();
        let mut sampler = self.hand_maker.sampler(bot.void);
        let mut samples = 0;
        for _ in 0..self.config.samples {
            if samples > 0
                && self
                    .config
                    .time_limit
                    .map_or(false, |limit| start.elapsed() >= limit)
            {
                break;
            }
            let hands = sampler.sample(&mut self.rng);
            let rollout_seed = self.rng.gen();
            let exact = 52 - state.played.len() <= self.config.exact_cards;
            for (i, &card) in plays.iter().enumerate() {
                let event = GameEvent::Play {
                    seat: bot.seat,
                    card,
                };
                let void = bot.void.on_event(state, &event);
                let mut game = state.clone();
                game.apply(&event);
                let won = if exact {
                    solve_endgame(&game, &hands)
                } else {
                    rollout(
                        &mut game,
                        void,
                        hands,
                        bot.seat,
                        None,
                        self.config.opponent_moon,
                        &mut StdRng::seed_from_u64(rollout_seed),
                    );
                    game.won
                };
                totals[i] += won.scores(state.charges).money(bot.seat) as f32;
                if try_moon {
                    let mut game = state.clone();
                    game.apply(&event);
                    rollout(
                        &mut game,
                        void,
                        hands,
                        bot.seat,
                        Some(bot.seat),
                        self.config.opponent_moon,
                        &mut StdRng::seed_from_u64(rollout_seed),
                    );
                    moon_totals[i] += game.scores().money(bot.seat) as f32;
                }
            }
            samples += 1;
        }
        // Choose a continuation using its mean across the same hidden deals.
        // Maximizing separately inside each deal would give the bot information
        // it does not have and badly overestimate speculative moon attempts.
        if try_moon {
            for i in 0..totals.len() {
                totals[i] = totals[i].max(moon_totals[i]);
            }
        }
        (
            plays
                .into_iter()
                .zip(totals.into_iter().map(|v| v / samples as f32))
                .collect(),
            samples,
        )
    }
}

impl Default for NeuralNetworkBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Algorithm for NeuralNetworkBot {
    fn pass(&mut self, bot: &BotState, state: &GameState) -> Cards {
        HeuristicBot.pass(bot, state)
    }

    fn charge(&mut self, bot: &BotState, state: &GameState) -> Cards {
        HeuristicBot.charge_with_rng(bot, state, &mut self.charge_rng)
    }

    fn play(&mut self, bot: &BotState, state: &GameState) -> Card {
        let legal = state.legal_plays(bot.post_pass_hand);
        let distinct = legal.distinct_plays(state.played, state.current_trick);
        if distinct.len() == 1 {
            return distinct.max();
        }
        let (mut values, samples) = self.search(bot, state);
        if let Some(policy) = &self.policy {
            if 52 - state.played.len() > self.config.exact_cards {
                for (card, value) in &mut values {
                    let prediction = policy.predict(&policy_features(bot, state, *card));
                    if prediction.is_finite() {
                        *value += self.prior_samples / samples as f32 * prediction;
                    }
                }
            }
        }
        values
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0
    }

    fn on_event(&mut self, _: &BotState, state: &GameState, event: &GameEvent) {
        self.hand_maker.on_event(state, event);
    }
}

fn rollout<R: Rng + ?Sized>(
    state: &mut GameState,
    mut void: VoidState,
    hands: [Cards; 4],
    root_seat: Seat,
    moon_seat: Option<Seat>,
    model_opponent_moon: bool,
    rng: &mut R,
) {
    let opponent_moon = if model_opponent_moon && rng.gen_bool(0.5) {
        Seat::VALUES.iter().copied().find(|s| {
            *s != root_seat
                && state.won.can_run(*s)
                && (state.won.hearts(*s) >= 3 || state.won.queen(*s))
        })
    } else {
        None
    };
    while state.played != Cards::ALL {
        if state.current_trick.is_empty() && state.played.contains_all(Cards::SCORING) {
            break;
        }
        let seat = state.next_actor.unwrap();
        let bot = BotState::with_void(seat, hands[seat.idx()], void);
        let card = if (moon_seat == Some(seat) || opponent_moon == Some(seat))
            && state.won.can_run(seat)
        {
            GottaTryBot.play_with_rng(&bot, state, rng)
        } else {
            HeuristicBot.play_with_rng(&bot, state, rng)
        };
        let event = GameEvent::Play { seat, card };
        void = void.on_event(state, &event);
        state.apply(&event);
    }
}

// With a small number of cards per player a direct max-n search is cheaper than
// constructing the legacy solver's 32,768-entry suit transposition tables.
fn solve_endgame(state: &GameState, hands: &[Cards; 4]) -> WonState {
    if state.played == Cards::ALL
        || (state.current_trick.is_empty() && state.played.contains_all(Cards::SCORING))
    {
        return state.won;
    }
    let seat = state.next_actor.unwrap();
    let mut best = state.won;
    let mut best_money = i16::MIN;
    for card in state
        .legal_plays(hands[seat.idx()])
        .distinct_plays(state.played, state.current_trick)
    {
        let mut child = state.clone();
        child.apply(&GameEvent::Play { seat, card });
        let won = solve_endgame(&child, hands);
        let money = won.scores(state.charges).money(seat);
        if money > best_money {
            best = won;
            best_money = money;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_hearts_api::{GamePhase, PassDirection, Suit, Suits, Trick};

    fn moon_position(can_finish: bool) -> ([Cards; 4], GameState) {
        // East has led KS. North can dump QS or win AS and keep the queen.
        // AH guarantees the remaining tricks; 2H guarantees a failed moon.
        let remaining: [Cards; 4] = if can_finish {
            ["AQS AH", "KS KH 2C", "2S QH 3C", "3S JH 4C"]
        } else {
            ["AQS 2H", "K2S AH", "3S KH 3C", "4S QH 4C"]
        }
        .map(|s| s.parse().unwrap());
        let all_remaining = remaining.iter().fold(Cards::NONE, |all, h| all | *h);
        let previous = Cards::ALL - all_remaining;
        let mut cards = previous.into_iter();
        let hands =
            std::array::from_fn(|i| remaining[i] | cards.by_ref().take(10).collect::<Cards>());
        let mut state = GameState::new();
        state.phase = GamePhase::PlayLeft;
        state.current_trick = Trick::new()
            .push(Card::KingSpades)
            .push((remaining[2] & Cards::SPADES).max())
            .push((remaining[3] & Cards::SPADES).max());
        state.played = previous | state.current_trick.cards();
        state.next_actor = Some(Seat::North);
        state.won = WonState::new()
            .win(Seat::North, previous & Cards::HEARTS)
            .win(Seat::East, Cards::JACK_DIAMONDS | Card::TenClubs);
        state.led_suits = Suits::NONE | Suit::Clubs | Suit::Diamonds | Suit::Spades | Suit::Hearts;
        (hands, state)
    }

    fn configured(hands: [Cards; 4], moon: bool) -> NeuralNetworkBot {
        let mut bot = NeuralNetworkBot::with_config(
            SearchConfig {
                samples: 32,
                exact_cards: 0,
                moon,
                opponent_moon: false,
                time_limit: None,
                ..SearchConfig::default()
            },
            123,
        );
        bot.hand_maker.on_event(
            &GameState::new(),
            &GameEvent::Deal {
                north: hands[0],
                east: hands[1],
                south: hands[2],
                west: hands[3],
                pass: PassDirection::Left,
            },
        );
        bot
    }

    #[test]
    fn endgames_match_the_existing_full_solver() {
        use rand::seq::SliceRandom;
        let mut rng = StdRng::seed_from_u64(82);
        for deal in 0..12 {
            let mut deck: Vec<_> = Cards::ALL.into_iter().collect();
            deck.shuffle(&mut rng);
            let hands: [Cards; 4] =
                std::array::from_fn(|i| deck[13 * i..13 * (i + 1)].iter().copied().collect());
            let mut state = GameState::new();
            state.phase = GamePhase::PlayLeft;
            if deal % 2 == 0 {
                for seat in Seat::VALUES {
                    state.charges = state
                        .charges
                        .charge(seat, hands[seat.idx()] & Cards::CHARGEABLE);
                }
            }
            state.next_actor = Seat::VALUES
                .iter()
                .copied()
                .find(|s| hands[s.idx()].contains(Card::TwoClubs));
            while state.played.len() < 40 {
                let seat = state.next_actor.unwrap();
                let legal = state.legal_plays(hands[seat.idx()]);
                let card = legal
                    .into_iter()
                    .nth(rng.gen_range(0..legal.len()))
                    .unwrap();
                state.apply(&GameEvent::Play { seat, card });
            }
            let expected = crate::BruteForce::new(hands).solve(&mut state.clone());
            assert_eq!(solve_endgame(&state, &hands), expected);
        }
    }

    #[test]
    fn shoots_a_certain_moon_but_dumps_the_queen_when_it_cannot_finish() {
        for (can_finish, expected) in [(true, Card::AceSpades), (false, Card::QueenSpades)] {
            let (hands, state) = moon_position(can_finish);
            let bot_state = BotState::new(Seat::North, hands[0]);
            assert!(state.won.can_run(Seat::North));
            assert_eq!(configured(hands, true).play(&bot_state, &state), expected);
        }
    }

    #[test]
    fn stops_considering_moons_after_an_opponent_wins_a_heart() {
        let (hands, mut state) = moon_position(true);
        let hearts = state.played & Cards::HEARTS;
        state.won = WonState::new()
            .win(Seat::North, hearts - Card::TwoHearts)
            .win(Seat::East, Card::TwoHearts.into());
        let bot_state = BotState::new(Seat::North, hands[0]);
        assert!(!state.won.can_run(Seat::North));
        assert_eq!(
            configured(hands, true).evaluate(&bot_state, &state),
            configured(hands, false).evaluate(&bot_state, &state)
        );
    }
}
