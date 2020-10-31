use crate::{Card, GameEvent, GameState, Seat, Suit};

#[derive(Clone, Copy, Debug)]
pub struct VoidState {
    state: u16,
}

impl VoidState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    fn mark_void(&mut self, seat: Seat, suit: Suit) {
        self.state |= 1 << (4 * seat.idx() + suit.idx())
    }

    pub fn is_void(&self, seat: Seat, suit: Suit) -> bool {
        self.state & (1 << (4 * seat.idx() + suit.idx())) != 0
    }

    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Play { seat, card } => {
                let trick = &state.current_trick;
                if !trick.is_empty() && trick.suit() != card.suit() {
                    // didn't follow suit
                    self.mark_void(*seat, trick.suit());
                } else if !state.led_suits.contains(card.suit()) {
                    if trick.is_empty() && card.suit() == Suit::Hearts {
                        // force break hearts
                        self.mark_void(*seat, Suit::Clubs);
                        self.mark_void(*seat, Suit::Diamonds);
                        self.mark_void(*seat, Suit::Spades);
                    } else if state.charges.is_charged(*card) {
                        // forced to play charged card
                        self.mark_void(*seat, card.suit());
                        if trick.is_empty() {
                            if *card == Card::QueenSpades {
                                // forced to lead charged queen
                                self.mark_void(*seat, Suit::Clubs);
                                self.mark_void(*seat, Suit::Diamonds);
                            }
                            if *card == Card::JackDiamonds {
                                // forced to lead charged jack
                                self.mark_void(*seat, Suit::Clubs);
                                if !state.charges.charges(*seat).contains(Card::QueenSpades) {
                                    self.mark_void(*seat, Suit::Spades);
                                }
                            }
                        }
                    }
                }
            }
            GameEvent::HandComplete { .. } => {
                self.state = 0;
            }
            _ => {}
        }
    }
}
