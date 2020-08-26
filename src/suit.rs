use crate::{card::Card, cards::Cards, rank::Rank};
use std::{
    convert::TryFrom,
    fmt,
    fmt::{Debug, Display, Write},
    mem,
};

const SUITS: [char; 4] = ['C', 'D', 'H', 'S'];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const VALUES: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn char(self) -> char {
        SUITS[self.idx()]
    }

    pub fn cards(self) -> Cards {
        Cards {
            bits: 0x1fff << (16 * self as u64),
        }
    }

    pub fn with_rank(self, rank: Rank) -> Card {
        Card::new(rank, self)
    }
}

impl From<u8> for Suit {
    fn from(n: u8) -> Self {
        assert!(n < 4, "n={}", n);
        unsafe { mem::transmute(n) }
    }
}

impl TryFrom<char> for Suit {
    type Error = char;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        SUITS
            .iter()
            .position(|&s| s == c)
            .map(|n| Self::from(n as u8))
            .ok_or(c)
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.char())
    }
}

impl Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}
