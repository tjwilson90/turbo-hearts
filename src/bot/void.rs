use crate::{
    game::{event::GameEvent, state::GameState},
    seat::Seat,
    suit::Suit,
};

#[derive(Clone, Copy, Debug)]
pub struct VoidState {
    state: u16,
}

impl VoidState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn is_void(&self, seat: Seat, suit: Suit) -> bool {
        self.state & (1 << (4 * seat.idx() + suit.idx())) != 0
    }

    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Play { seat, card } => {
                let suit = state.current_trick.suit();
                if card.suit() != suit {
                    self.state |= 1 << (4 * seat.idx() + suit.idx());
                }
            }
            GameEvent::HandComplete { .. } => {
                self.state = 0;
            }
            _ => {}
        }
    }
}
