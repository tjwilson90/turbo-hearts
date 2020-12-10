use crate::{Algorithm, BruteForce, HandMaker, HeuristicBot, VoidState};
use log::debug;
use rand::Rng;
use std::{collections::HashMap, fmt::Display, hash::Hash, time::Instant};
use turbo_hearts_api::{can_claim, BotState, Card, Cards, GameEvent, GameState, Seat};

#[derive(Clone)]
pub struct SimulateBot {
    void: VoidState,
}

impl SimulateBot {
    pub fn new() -> Self {
        Self {
            void: VoidState::new(),
        }
    }

    fn heuristic_bot(&mut self) -> HeuristicBot {
        HeuristicBot::from(self.void.clone())
    }

    fn charge_blocking(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        let chargeable =
            (bot_state.post_pass_hand - game_state.charges.all_charges()) & Cards::CHARGEABLE;
        let hand_maker = HandMaker::new(&bot_state, &game_state, self.void.clone());
        let mut money_counts = HashMap::new();
        let now = Instant::now();
        let deadline = 4000 + rand::thread_rng().gen_range(0, 1000);
        while now.elapsed().as_millis() < deadline {
            let hands = hand_maker.make();
            for cards in chargeable.powerset() {
                let mut bot = self.heuristic_bot();
                let mut game = game_state.clone();
                game.apply(&GameEvent::Charge {
                    seat: bot_state.seat,
                    cards,
                });
                do_charges(&mut bot, &mut game, hands);
                do_passes(&mut game);
                do_charges(&mut bot, &mut game, hands);
                for &seat in &Seat::VALUES {
                    if hands[seat.idx()].contains(Card::TwoClubs) {
                        game.next_actor = Some(seat);
                        break;
                    }
                }
                do_plays(&mut bot, &mut game, hands);
                let money = money(&bot_state, &game);
                *money_counts.entry((cards, money)).or_default() += 1;
            }
        }
        compute_best(chargeable.powerset(), money_counts)
    }

    fn play_blocking(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if cards.contains(Card::TwoClubs) {
            return Card::TwoClubs;
        }
        let hand_maker = HandMaker::new(&bot_state, &game_state, self.void.clone());
        let mut money_counts = HashMap::new();
        let now = Instant::now();
        let mut iter = 0;
        while now.elapsed().as_millis() < 3500 {
            let hands = hand_maker.make();
            for card in cards {
                let mut game = game_state.clone();
                game.apply(&GameEvent::Play {
                    seat: bot_state.seat,
                    card,
                });
                if game.played.len() > 28 && iter > 0 {
                    let mut brute_force = BruteForce::new(hands);
                    let (_, won) = brute_force.solve(&mut game);
                    *money_counts
                        .entry((
                            card,
                            won.scores(game.charges.all_charges()).money(bot_state.seat),
                        ))
                        .or_default() += 1;
                } else {
                    for _ in 0..50 {
                        let mut game = game.clone();
                        do_plays(&mut self.heuristic_bot(), &mut game, hands);
                        let money = money(&bot_state, &game);
                        *money_counts.entry((card, money)).or_default() += 1;
                    }
                };
            }
            iter += 1;
        }
        compute_best(cards, money_counts)
    }
}

impl Algorithm for SimulateBot {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        self.heuristic_bot().pass(bot_state, game_state)
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        tokio::task::block_in_place(move || self.charge_blocking(&bot_state, &game_state))
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        tokio::task::block_in_place(move || self.play_blocking(&bot_state, &game_state))
    }

    fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.void.on_event(game_state, event);
    }
}

fn do_passes(game: &mut GameState) {
    if game.phase.is_passing() {
        for &seat in &Seat::VALUES {
            game.apply(&GameEvent::RecvPass {
                to: seat,
                cards: Cards::NONE,
            });
        }
    }
}

fn do_charges(bot: &mut HeuristicBot, game: &mut GameState, hands: [Cards; 4]) {
    while game.phase.is_charging() {
        for &seat in &Seat::VALUES {
            if !game.done.charged(seat) {
                let cards = bot.charge(
                    &BotState {
                        seat,
                        pre_pass_hand: hands[seat.idx()],
                        post_pass_hand: hands[seat.idx()],
                    },
                    &game,
                );
                game.apply(&GameEvent::Charge { seat, cards });
            }
        }
    }
}

fn do_plays(bot: &mut HeuristicBot, game: &mut GameState, hands: [Cards; 4]) {
    while game.phase.is_playing() {
        let seat = game.next_actor.unwrap();
        if game.current_trick.is_empty()
            && game.won.can_run(seat)
            && can_claim(seat, hands[seat.idx()], game)
        {
            game.won = game.won.win(seat, Cards::ALL - game.played);
            return;
        }
        let card = bot.play(
            &BotState {
                seat,
                pre_pass_hand: hands[seat.idx()],
                post_pass_hand: hands[seat.idx()],
            },
            &game,
        );
        game.apply(&GameEvent::Play { seat, card });
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

#[cfg(test)]
mod test {
    use super::*;
    use turbo_hearts_api::{
        ChargeState, ChargingRules, ClaimState, DoneState, GamePhase, Suit, Suits, Trick, WonState,
    };

    #[test]
    fn test_make_hands() {
        let mut simulate = SimulateBot::new();
        simulate.void.mark_void(Seat::East, Suit::Spades);
        simulate.void.mark_void(Seat::North, Suit::Diamonds);
        simulate.void.mark_void(Seat::East, Suit::Clubs);
        simulate.void.mark_void(Seat::South, Suit::Clubs);
        simulate.void.mark_void(Seat::North, Suit::Spades);
        simulate.void.mark_void(Seat::South, Suit::Spades);
        let bot_state = BotState {
            seat: Seat::North,
            pre_pass_hand: "J7S AK984H 3D AQ985C".parse().unwrap(),
            post_pass_hand: "J7S Q9864H AKQ985C".parse().unwrap(),
        };
        let game_state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassLeft,
            done: DoneState::new(),
            charge_count: 2,
            charges: ChargeState::new().charge(Seat::East, Card::AceHearts | Card::JackDiamonds),
            next_actor: Some(Seat::North),
            played:
                "2JA3C J95473TS AH 4KJTH 457D 9H 6JKD 8H 4T9C AD QS 78C QD 65C 2D 8S AS QH TD 7H KS"
                    .parse()
                    .unwrap(),
            claims: ClaimState::new(),
            won: WonState::new(),
            led_suits: Suits::NONE,
            current_trick: Trick::new().push(Card::KingSpades),
        };
        let hand_maker = HandMaker::new(&bot_state, &game_state, simulate.void.clone());
        for _ in 0..100 {
            let hands = hand_maker.make();
            eprintln!("{:?}", hands);
        }
    }
}
