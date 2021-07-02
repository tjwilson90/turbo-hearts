use crate::{Algorithm, BruteForce, HandMaker, HeuristicBot};
use log::debug;
use std::{collections::HashMap, fmt::Display, hash::Hash, time::Instant};
use turbo_hearts_api::{can_claim, BotState, Card, Cards, GameEvent, GameState, VoidState};

#[derive(Clone)]
pub struct SimulateBot {
    hand_maker: HandMaker,
}

impl SimulateBot {
    pub fn new() -> Self {
        Self {
            hand_maker: HandMaker::new(),
        }
    }
}

impl Algorithm for SimulateBot {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot.pass(bot_state, game_state)
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        HeuristicBot.charge(bot_state, game_state)
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if cards.contains(Card::TwoClubs) {
            return Card::TwoClubs;
        }
        let mut money_counts = HashMap::new();
        let now = Instant::now();
        let mut iter = 0;
        while now.elapsed().as_millis() < 3500 {
            let hands = self.hand_maker.make(&bot_state.void);
            for card in cards {
                let event = GameEvent::Play {
                    seat: bot_state.seat,
                    card,
                };
                let void = bot_state.void.on_event(&game_state, &event);
                let mut game = game_state.clone();
                game.apply(&event);
                if game.played.len() > 28 && iter > 0 {
                    let mut brute_force = BruteForce::new(hands);
                    let won = brute_force.solve(&mut game);
                    *money_counts
                        .entry((card, won.scores(game.charges).money(bot_state.seat)))
                        .or_default() += 1;
                } else {
                    for _ in 0..50 {
                        let mut game = game.clone();
                        do_plays(&mut game, void, hands);
                        let money = money(&bot_state, &game);
                        *money_counts.entry((card, money)).or_default() += 1;
                    }
                };
            }
            iter += 1;
        }
        compute_best(cards, money_counts)
    }

    fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.hand_maker.on_event(game_state, event);
    }
}

fn do_plays(game: &mut GameState, mut void: VoidState, hands: [Cards; 4]) {
    while game.phase.is_playing() {
        let seat = game.next_actor.unwrap();
        if game.current_trick.is_empty()
            && game.won.can_run(seat)
            && can_claim(game, &void, seat, hands[seat.idx()])
        {
            game.won = game.won.win(seat, Cards::ALL - game.played);
            return;
        }
        let card = HeuristicBot.play(&BotState::with_void(seat, hands[seat.idx()], void), &game);
        let event = GameEvent::Play { seat, card };
        void = void.on_event(&game, &event);
        game.apply(&event);
    }
}

fn money(bot_state: &BotState, game: &GameState) -> i16 {
    let scores = game.scores();
    scores.money(bot_state.seat)
}

fn compute_best<T, I>(choices: I, money_counts: HashMap<(T, i16), u32>) -> T
where
    T: Copy + Display + Eq + Hash,
    I: IntoIterator<Item = T>,
{
    let mut means: HashMap<T, (i64, u32)> = HashMap::new();
    for ((choice, money), &count) in money_counts.iter() {
        let (total, cnt) = means.entry(*choice).or_default();
        *total += *money as i64 * count as i64;
        *cnt += count;
    }
    let mut vars: HashMap<T, f32> = HashMap::new();
    for ((choice, money), &count) in money_counts.iter() {
        let (total, cnt) = means[choice];
        let mean = total as f32 / cnt as f32;
        let var = vars.entry(*choice).or_default();
        *var += count as f32 * (*money as f32 - mean).powi(2);
    }
    let mut best = None;
    let mut best_score = -10000.0;
    for choice in choices {
        let (total, cnt) = means[&choice];
        let mean = total as f32 / cnt as f32;
        let stddev = if cnt == 1 {
            0.0
        } else {
            (vars[&choice] / (cnt as f32 - 1.0)).sqrt()
        };
        debug!(
            "{}: cnt={}, mean={:.2}, stddev={:.2}",
            choice, cnt, mean, stddev
        );
        let score = mean - stddev / 5.0;
        if score > best_score {
            best = Some(choice);
            best_score = score;
        }
    }
    best.unwrap()
}
