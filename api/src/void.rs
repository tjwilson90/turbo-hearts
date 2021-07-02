use crate::{Card, Cards, GameEvent, GameState, Seat, Suit};
use std::{fmt, fmt::Formatter};

#[derive(Clone, Copy)]
pub struct VoidState {
    state: u16,
}

impl VoidState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub fn mark_void(self, seat: Seat, suit: Suit) -> VoidState {
        Self {
            state: self.state | 1 << (4 * seat.idx() + suit.idx()),
        }
    }

    pub fn is_void(self, seat: Seat, suit: Suit) -> bool {
        self.state & (1 << (4 * seat.idx() + suit.idx())) != 0
    }

    #[must_use]
    pub fn on_event(self, state: &GameState, event: &GameEvent) -> VoidState {
        match event {
            GameEvent::Play { seat, card } => {
                let trick = state.current_trick;
                if !trick.is_empty() && trick.suit() != card.suit() {
                    // didn't follow suit
                    self.mark_void(*seat, trick.suit())
                } else if trick.is_empty()
                    && card.suit() == Suit::Hearts
                    && !state.played.contains_any(Cards::HEARTS)
                {
                    // force break hearts
                    self.mark_void(*seat, Suit::Clubs)
                        .mark_void(*seat, Suit::Diamonds)
                        .mark_void(*seat, Suit::Spades)
                } else if !state.led_suits.contains(card.suit()) && state.charges.is_charged(*card)
                {
                    // forced to play charged card
                    let mut void = self.mark_void(*seat, card.suit());
                    if trick.is_empty() {
                        if *card == Card::QueenSpades {
                            // forced to lead charged queen
                            void = void.mark_void(*seat, Suit::Clubs);
                            void = void.mark_void(*seat, Suit::Diamonds);
                        }
                        if *card == Card::JackDiamonds {
                            // forced to lead charged jack
                            void = void.mark_void(*seat, Suit::Clubs);
                            if !state.charges.charges(*seat).contains(Card::QueenSpades) {
                                void = void.mark_void(*seat, Suit::Spades);
                            }
                        }
                    }
                    void
                } else {
                    self
                }
            }
            GameEvent::HandComplete { .. } => Self::new(),
            _ => self,
        }
    }
}

impl fmt::Debug for VoidState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for &seat in &Seat::VALUES {
            if seat != Seat::North {
                write!(f, ", ")?;
            }
            write!(f, "{} [", seat)?;
            for &suit in &Suit::VALUES {
                if self.is_void(seat, suit) {
                    write!(f, "{}", suit.char())?;
                }
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
