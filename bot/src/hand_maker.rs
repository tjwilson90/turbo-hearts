use crate::VoidState;
use rand::seq::SliceRandom;
use turbo_hearts_api::{BotState, Card, Cards, GameState, PassDirection, Seat, Suit};

#[derive(Clone, Debug)]
pub struct HandMaker {
    hands: [Cards; 4],
    sizes: [usize; 4],
    void: VoidState,
    cards: Vec<Card>,
    unassigned: Cards,
}

impl HandMaker {
    pub fn new(bot_state: &BotState, game_state: &GameState, void: VoidState) -> Self {
        let mut hands = [Cards::NONE; 4];
        hands[bot_state.seat.idx()] = bot_state.post_pass_hand;
        if game_state.phase.direction() != PassDirection::Keeper {
            let receiver = game_state.phase.pass_receiver(bot_state.seat);
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
        let cards = unassigned.into_iter().collect::<Vec<_>>();
        Self {
            hands,
            sizes,
            void,
            cards,
            unassigned,
        }
    }

    pub fn known_cards(&self, seat: Seat) -> Cards {
        self.hands[seat.idx()]
    }

    pub fn make(&self) -> [Cards; 4] {
        let mut copy = self.clone();
        copy.cards.shuffle(&mut rand::thread_rng());
        assert!(copy.assign(), "failed hand assignment for {:?}", self);
        copy.hands
    }

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
