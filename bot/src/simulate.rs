use crate::{HeuristicBot, VoidState};
use log::debug;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use std::{collections::HashMap, fmt::Display, hash::Hash, time::Instant};
use tokio::task;
use turbo_hearts_api::{can_claim, BotState, Card, Cards, GameEvent, GameState, Seat, Suit};

pub struct SimulateBot {
    void: VoidState,
    rng: SmallRng,
}

impl SimulateBot {
    pub fn new() -> Self {
        Self {
            void: VoidState::new(),
            rng: SmallRng::from_rng(rand::thread_rng()).unwrap(),
        }
    }

    pub async fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        HeuristicBot::pass_sync(bot_state)
    }

    pub async fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        let mut bot = HeuristicBot::from(self.void, self.rng.clone());
        let chargeable =
            (bot_state.post_pass_hand - game_state.charges.all_charges()) & Cards::CHARGEABLE;
        let mut money_counts = HashMap::new();
        let now = Instant::now();
        let deadline = 4000 + self.rng.gen_range(0, 1000);
        while now.elapsed().as_millis() < deadline {
            let now = Instant::now();
            while now.elapsed().as_millis() < 10 {
                let hands = self.make_hands(bot_state, game_state);
                for cards in chargeable.powerset() {
                    let mut worst_money = i16::max_value();
                    for _ in 0..20 {
                        let mut game = game_state.clone();
                        game.apply(&GameEvent::Charge {
                            seat: bot_state.seat,
                            cards,
                        });
                        do_charges(&mut game, hands);
                        do_passes(&mut game);
                        do_charges(&mut game, hands);
                        for &seat in &Seat::VALUES {
                            if hands[seat.idx()].contains(Card::TwoClubs) {
                                game.next_actor = Some(seat);
                                break;
                            }
                        }
                        do_plays(&mut game, hands, &mut bot);
                        worst_money = worst_money.min(money(&bot_state, &game));
                    }
                    *money_counts.entry((cards, worst_money)).or_default() += 1;
                }
            }
            task::yield_now().await;
        }
        compute_best(chargeable.powerset(), money_counts)
    }

    pub async fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if cards.contains(Card::TwoClubs) {
            return Card::TwoClubs;
        }
        let mut bot = HeuristicBot::from(self.void, self.rng.clone());
        let mut money_counts = HashMap::new();
        let now = Instant::now();
        while now.elapsed().as_millis() < 3500 {
            let now = Instant::now();
            while now.elapsed().as_millis() < 10 {
                let hands = self.make_hands(bot_state, game_state);
                for card in cards {
                    let mut worst_money = i16::max_value();
                    let mut game = game_state.clone();
                    game.apply(&GameEvent::Play {
                        seat: bot_state.seat,
                        card,
                    });
                    for _ in 0..50 {
                        let mut game = game.clone();
                        do_plays(&mut game, hands, &mut bot);
                        worst_money = worst_money.min(money(&bot_state, &game));
                    }
                    *money_counts.entry((card, worst_money)).or_default() += 1;
                }
            }
            task::yield_now().await;
        }
        compute_best(cards, money_counts)
    }

    pub fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.void.on_event(game_state, event);
    }

    fn make_hands(&mut self, bot_state: &BotState, game_state: &GameState) -> [Cards; 4] {
        let mut hands = [Cards::NONE; 4];
        hands[bot_state.seat.idx()] = bot_state.post_pass_hand;
        let receiver = game_state.phase.pass_receiver(bot_state.seat);
        if receiver != bot_state.seat {
            hands[receiver.idx()] |= bot_state.pre_pass_hand - bot_state.post_pass_hand;
        }
        for &seat in &Seat::VALUES {
            hands[seat.idx()] |= game_state.charges.charges(seat);
            hands[seat.idx()] -= game_state.played;
        }
        let unplayed = Cards::ALL - game_state.played;
        let mut sizes = [unplayed.len() / 4; 4];
        let additions = unplayed.len() % 4;
        if additions >= 1 {
            sizes[bot_state.seat.idx()] += 1;
        }
        if additions >= 2 {
            sizes[bot_state.seat.left().idx()] += 1;
        }
        if additions >= 3 {
            sizes[bot_state.seat.across().idx()] += 1;
        }
        let unassigned = Cards::ALL - hands[0] - hands[1] - hands[2] - hands[3] - game_state.played;
        let mut cards = unassigned.into_iter().collect::<Vec<_>>();
        cards.shuffle(&mut self.rng);
        let mut state = State {
            hands,
            sizes,
            void: self.void,
            cards,
            unassigned,
        };
        state.assign();
        state.hands
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

fn do_charges(game: &mut GameState, hands: [Cards; 4]) {
    while game.phase.is_charging() {
        for &seat in &Seat::VALUES {
            if !game.done.charged(seat) {
                let cards = HeuristicBot::charge_sync(
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

fn do_plays(game: &mut GameState, hands: [Cards; 4], bot: &mut HeuristicBot) {
    while game.phase.is_playing() {
        let seat = game.next_actor.unwrap();
        if game.current_trick.is_empty()
            && game.won.can_run(seat)
            && can_claim(seat, hands[seat.idx()], game)
        {
            game.won.win(seat, Cards::ALL - game.played);
            return;
        }
        let card = bot.play_sync(
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
    let scores = [
        game.score(Seat::North),
        game.score(Seat::East),
        game.score(Seat::South),
        game.score(Seat::West),
    ];
    scores[0] + scores[1] + scores[2] + scores[3] - 4 * scores[bot_state.seat.idx()]
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
        let stddev = (vars[&choice] / (cnt as f32 - 1.0)).sqrt();
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

#[derive(Debug)]
struct State {
    hands: [Cards; 4],
    sizes: [usize; 4],
    void: VoidState,
    cards: Vec<Card>,
    unassigned: Cards,
}

impl State {
    fn assign(&mut self) -> bool {
        if self.cards.is_empty() {
            return true;
        }
        for &seat in &Seat::VALUES {
            let mut available = 0;
            for &suit in &Suit::VALUES {
                if !self.void.is_void(seat, suit) {
                    available += (self.unassigned & suit.cards()).len();
                }
            }
            let holes = self.sizes[seat.idx()] - self.hands[seat.idx()].len();
            if available < holes {
                return false;
            }
        }
        let card = self.cards.pop().unwrap();
        self.unassigned -= card;
        for &seat in &Seat::VALUES {
            if self.hands[seat.idx()].len() >= self.sizes[seat.idx()] {
                continue;
            }
            if self.void.is_void(seat, card.suit()) {
                continue;
            }
            self.hands[seat.idx()] |= card;
            if self.assign() {
                return true;
            }
            self.hands[seat.idx()] -= card;
        }
        self.cards.push(card);
        self.unassigned |= card;
        false
    }
}
