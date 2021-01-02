use crate::VoidState;
use rand::seq::SliceRandom;
use turbo_hearts_api::{Card, Cards, GameEvent, GameState, Seat, Suit};

#[derive(Clone, Debug)]
pub struct HandMaker {
    hands: [Cards; 4],
    void: VoidState,
}

impl HandMaker {
    pub fn new() -> Self {
        Self {
            hands: [Cards::NONE; 4],
            void: VoidState::new(),
        }
    }

    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        self.void.on_event(state, event);
        match event {
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.hands[0] = *north;
                self.hands[1] = *east;
                self.hands[2] = *south;
                self.hands[3] = *west;
            }
            GameEvent::SendPass { from, cards } => {
                self.hands[from.idx()] -= *cards;
                let recv = state.phase.pass_receiver(*from);
                if *from != recv {
                    self.hands[recv.idx()] |= *cards;
                }
            }
            GameEvent::RecvPass { to, cards } => self.hands[to.idx()] |= *cards,
            GameEvent::Charge { seat, cards } => self.hands[seat.idx()] |= *cards,
            GameEvent::RevealCharges {
                north,
                east,
                south,
                west,
            } => {
                self.hands[0] |= *north;
                self.hands[1] |= *east;
                self.hands[2] |= *south;
                self.hands[3] |= *west;
            }
            GameEvent::Play { seat, card } => self.hands[seat.idx()] |= *card,
            GameEvent::HandComplete { .. } => self.hands = [Cards::NONE; 4],
            _ => {}
        }
    }

    pub fn void(&self) -> VoidState {
        self.void.clone()
    }

    pub fn known_cards(&self, seat: Seat) -> Cards {
        self.hands[seat.idx()]
    }

    pub fn make(&self) -> [Cards; 4] {
        let mut copy = self.clone();
        let cards = Cards::ALL - self.hands[0] - self.hands[1] - self.hands[2] - self.hands[3];
        let mut shuffled = cards.into_iter().collect::<Vec<_>>();
        shuffled.shuffle(&mut rand::thread_rng());
        assert!(
            copy.assign(&mut shuffled, cards),
            "failed hand assignment for {:?}",
            self
        );
        copy.hands
    }

    fn assign(&mut self, shuffled: &mut Vec<Card>, cards: Cards) -> bool {
        if shuffled.is_empty() {
            return true;
        }
        for &seat in &Seat::VALUES {
            let mut available = 0;
            for &suit in &Suit::VALUES {
                if !self.void.is_void(seat, suit) {
                    available += (cards & suit.cards()).len();
                }
            }
            if available + self.hands[seat.idx()].len() < 13 {
                return false;
            }
        }
        let card = shuffled.pop().unwrap();
        for &seat in &Seat::VALUES {
            if self.hands[seat.idx()].len() == 13 || self.void.is_void(seat, card.suit()) {
                continue;
            }
            self.hands[seat.idx()] |= card;
            if self.assign(shuffled, cards - card) {
                return true;
            }
            self.hands[seat.idx()] -= card;
        }
        shuffled.push(card);
        false
    }
}
