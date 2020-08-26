use crate::{
    bot::{heuristic::Heuristic, void::VoidState, Algorithm, BotState},
    card::Card,
    cards::Cards,
    game::{event::GameEvent, state::GameState},
    seat::Seat,
    suit::Suit,
};
use rand::seq::SliceRandom;
use std::{collections::HashMap, time::Instant};

pub struct Simulate {
    void: VoidState,
}

impl Simulate {
    pub fn new() -> Self {
        Self {
            void: VoidState::new(),
        }
    }
}

impl Algorithm for Simulate {
    fn pass(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        Heuristic::new().pass(bot_state, game_state)
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        Heuristic::new().charge(bot_state, game_state)
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        let mut dist: HashMap<Card, i64> = HashMap::new();
        let now = Instant::now();
        while now.elapsed().as_secs() < 3 {
            let hands = make_hands(bot_state, game_state, self.void);
            for _ in 0..100 {
                for card in cards {
                    let mut game: GameState = game_state.clone();
                    game.apply(&GameEvent::Play {
                        seat: bot_state.seat,
                        card,
                    });
                    while game.phase.is_playing() {
                        let seat = game.next_actor.unwrap();
                        let card = Heuristic::with_void(self.void).play(
                            &BotState {
                                seat,
                                pre_pass_hand: hands[seat.idx()],
                                post_pass_hand: hands[seat.idx()],
                            },
                            &game,
                        );
                        game.apply(&GameEvent::Play { seat, card });
                    }
                    let scores = [
                        game.score(Seat::North),
                        game.score(Seat::East),
                        game.score(Seat::South),
                        game.score(Seat::West),
                    ];
                    let money = scores[0] + scores[1] + scores[2] + scores[3]
                        - 4 * scores[bot_state.seat.idx()];
                    *dist.entry(card).or_default() += money as i64;
                }
            }
        }
        dist.into_iter().max_by_key(|(_, money)| *money).unwrap().0
    }

    fn on_event(&mut self, _: &BotState, game_state: &GameState, event: &GameEvent) {
        self.void.on_event(game_state, event);
    }
}

fn make_hands(bot_state: &BotState, game_state: &GameState, void: VoidState) -> [Cards; 4] {
    let mut hands = [Cards::NONE; 4];
    hands[bot_state.seat.idx()] = bot_state.post_pass_hand;
    let receiver = game_state.phase.pass_receiver(bot_state.seat);
    if receiver != bot_state.seat {
        hands[receiver.idx()] |= bot_state.pre_pass_hand - bot_state.post_pass_hand;
    }
    for seat in &Seat::VALUES {
        hands[seat.idx()] |= game_state.charges.charges(*seat);
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
    cards.shuffle(&mut rand::thread_rng());
    let mut state = State {
        hands,
        sizes,
        void,
        cards,
        unassigned,
    };
    state.assign();
    state.hands
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
        for seat in &Seat::VALUES {
            let mut available = 0;
            for suit in &Suit::VALUES {
                if !self.void.is_void(*seat, *suit) {
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
        for seat in &Seat::VALUES {
            if self.hands[seat.idx()].len() >= self.sizes[seat.idx()] {
                continue;
            }
            if self.void.is_void(*seat, card.suit()) {
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
