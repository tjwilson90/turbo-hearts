use crate::{Cards, Rank, Suit};
use serde::{Deserialize, Serialize};
use std::{
    convert::{Infallible, TryFrom},
    fmt,
    fmt::{Debug, Display, Write},
    mem,
    ops::BitOr,
    str::FromStr,
};

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(from = "String")]
#[serde(into = "String")]
pub enum Card {
    TwoClubs = 0,
    ThreeClubs,
    FourClubs,
    FiveClubs,
    SixClubs,
    SevenClubs,
    EightClubs,
    NineClubs,
    TenClubs,
    JackClubs,
    QueenClubs,
    KingClubs,
    AceClubs,
    TwoDiamonds = 16,
    ThreeDiamonds,
    FourDiamonds,
    FiveDiamonds,
    SixDiamonds,
    SevenDiamonds,
    EightDiamonds,
    NineDiamonds,
    TenDiamonds,
    JackDiamonds,
    QueenDiamonds,
    KingDiamonds,
    AceDiamonds,
    TwoHearts = 32,
    ThreeHearts,
    FourHearts,
    FiveHearts,
    SixHearts,
    SevenHearts,
    EightHearts,
    NineHearts,
    TenHearts,
    JackHearts,
    QueenHearts,
    KingHearts,
    AceHearts,
    TwoSpades = 48,
    ThreeSpades,
    FourSpades,
    FiveSpades,
    SixSpades,
    SevenSpades,
    EightSpades,
    NineSpades,
    TenSpades,
    JackSpades,
    QueenSpades,
    KingSpades,
    AceSpades,
}

impl Card {
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self::from(16 * suit as u8 + rank as u8)
    }

    pub fn rank(self) -> Rank {
        Rank::from(self as u8 % 16)
    }

    pub fn with_rank(self, rank: Rank) -> Card {
        Card::new(rank, self.suit())
    }

    pub fn suit(self) -> Suit {
        Suit::from(self as u8 / 16)
    }

    pub fn with_suit(self, suit: Suit) -> Card {
        Card::new(self.rank(), suit)
    }
}

impl From<u8> for Card {
    fn from(n: u8) -> Self {
        assert!(n < 64 && n % 16 < 13, "n={}", n);
        unsafe { mem::transmute(n) }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.rank().char())?;
        f.write_char(self.suit().char())
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl FromStr for Card {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let rank = Rank::try_from(chars.next().unwrap()).unwrap();
        let suit = Suit::try_from(chars.next().unwrap()).unwrap();
        Ok(Card::new(rank, suit))
    }
}

impl From<String> for Card {
    fn from(s: String) -> Self {
        Card::from_str(&s).unwrap()
    }
}

impl From<Card> for String {
    fn from(c: Card) -> Self {
        c.to_string()
    }
}

impl BitOr<Card> for Card {
    type Output = Cards;

    fn bitor(self, rhs: Card) -> Self::Output {
        Cards::from(self) | rhs
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Card::NineSpades.to_string(), "9S");
        assert_eq!(Card::ThreeDiamonds.to_string(), "3D");
        assert_eq!(Card::JackClubs.to_string(), "JC");
        assert_eq!(Card::AceHearts.to_string(), "AH");
    }

    #[test]
    fn test_suit() {
        assert_eq!(Card::TwoClubs.suit(), Suit::Clubs);
        assert_eq!(Card::AceClubs.suit(), Suit::Clubs);
        assert_eq!(Card::TwoDiamonds.suit(), Suit::Diamonds);
        assert_eq!(Card::AceDiamonds.suit(), Suit::Diamonds);
        assert_eq!(Card::TwoHearts.suit(), Suit::Hearts);
        assert_eq!(Card::AceHearts.suit(), Suit::Hearts);
        assert_eq!(Card::TwoSpades.suit(), Suit::Spades);
        assert_eq!(Card::AceSpades.suit(), Suit::Spades);
    }
}
