use crate::{Cards, Suit};
use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Suits {
    bits: u8,
}

impl Suits {
    pub const NONE: Suits = Suits { bits: 0x0 };

    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }

    pub fn contains(self, other: Suit) -> bool {
        self == self | other
    }

    pub fn cards(self) -> Cards {
        let mut cards = Cards::NONE;
        if self.contains(Suit::Clubs) {
            cards |= Suit::Clubs.cards();
        }
        if self.contains(Suit::Diamonds) {
            cards |= Suit::Diamonds.cards();
        }
        if self.contains(Suit::Hearts) {
            cards |= Suit::Hearts.cards();
        }
        if self.contains(Suit::Spades) {
            cards |= Suit::Spades.cards();
        }
        cards
    }
}

impl From<Suit> for Suits {
    fn from(suit: Suit) -> Self {
        Suits {
            bits: 1 << (suit as u8),
        }
    }
}

impl BitOr<Suits> for Suits {
    type Output = Self;

    fn bitor(self, rhs: Suits) -> Self::Output {
        Suits {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOr<Suit> for Suits {
    type Output = Self;

    fn bitor(self, rhs: Suit) -> Self::Output {
        self | Self::from(rhs)
    }
}

impl BitOrAssign<Suits> for Suits {
    fn bitor_assign(&mut self, rhs: Suits) {
        self.bits |= rhs.bits;
    }
}

impl BitOrAssign<Suit> for Suits {
    fn bitor_assign(&mut self, rhs: Suit) {
        *self |= Self::from(rhs)
    }
}
