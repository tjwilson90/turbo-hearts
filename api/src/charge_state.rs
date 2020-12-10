use crate::{Card, Cards, Seat};
use serde::export::Formatter;
use std::fmt;

#[derive(Copy, Clone)]
pub struct ChargeState {
    charges: u16,
}

impl ChargeState {
    pub fn new() -> Self {
        Self { charges: 0 }
    }

    #[must_use]
    pub fn charge(self, seat: Seat, cards: Cards) -> Self {
        let mut mask = 0;
        if !cards.is_empty() {
            if cards.contains(Card::QueenSpades) {
                mask += 8;
            }
            if cards.contains(Card::AceHearts) {
                mask += 4;
            }
            if cards.contains(Card::JackDiamonds) {
                mask += 2;
            }
            if cards.contains(Card::TenClubs) {
                mask += 1;
            }
        }
        Self {
            charges: self.charges | (mask << (4 * seat.idx())),
        }
    }

    pub fn is_charged(self, card: Card) -> bool {
        match card {
            Card::QueenSpades => self.charges & 0x8888 != 0,
            Card::AceHearts => self.charges & 0x4444 != 0,
            Card::JackDiamonds => self.charges & 0x2222 != 0,
            Card::TenClubs => self.charges & 0x1111 != 0,
            _ => false,
        }
    }

    pub fn charges(self, seat: Seat) -> Cards {
        self.charges_from_mask(self.charges & (0xf << (4 * seat.idx())))
    }

    pub fn all_charges(self) -> Cards {
        self.charges_from_mask(self.charges)
    }

    fn charges_from_mask(self, mask: u16) -> Cards {
        let mut charges = Cards::NONE;
        if mask & 0x8888 != 0 {
            charges |= Card::QueenSpades;
        }
        if mask & 0x4444 != 0 {
            charges |= Card::AceHearts;
        }
        if mask & 0x2222 != 0 {
            charges |= Card::JackDiamonds;
        }
        if mask & 0x1111 != 0 {
            charges |= Card::TenClubs;
        }
        charges
    }
}

impl fmt::Debug for ChargeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        for &seat in &Seat::VALUES {
            if seat != Seat::North {
                write!(f, ", ")?;
            }
            write!(f, "{} [{}]", seat, self.charges(seat))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_charges() {
        let state = ChargeState::new();
        assert_eq!(state.all_charges(), Cards::NONE);
        for &seat in &Seat::VALUES {
            assert_eq!(state.charges(seat), Cards::NONE);
        }
        for card in Cards::ALL {
            assert!(!state.is_charged(card));
        }
    }

    #[test]
    fn charge() {
        let state = ChargeState::new()
            .charge(Seat::North, Cards::QUEEN_SPADES)
            .charge(Seat::West, Card::TenClubs | Card::JackDiamonds);
        assert_eq!(state.all_charges(), Cards::CHARGEABLE - Card::AceHearts);
        assert_eq!(state.charges(Seat::North), Cards::QUEEN_SPADES);
        assert_eq!(state.charges(Seat::East), Cards::NONE);
        assert_eq!(state.charges(Seat::South), Cards::NONE);
        assert_eq!(
            state.charges(Seat::West),
            Card::TenClubs | Card::JackDiamonds
        );
        for card in Cards::ALL {
            if (Cards::CHARGEABLE - Card::AceHearts).contains(card) {
                assert!(state.is_charged(card));
            } else {
                assert!(!state.is_charged(card));
            }
        }
    }
}
