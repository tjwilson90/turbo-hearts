use crate::{Card, Rank, Suit};
use serde::{
    de::{SeqAccess, Visitor},
    export::{fmt::Error, Formatter},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    convert::{Infallible, TryFrom},
    fmt,
    fmt::{Debug, Display, Write},
    iter::FromIterator,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign},
    str::FromStr,
};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Cards {
    pub bits: u64,
}

impl Serialize for Cards {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for card in self {
            seq.serialize_element(&card)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Cards {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(CardsVisitor(Cards::NONE))
    }
}

struct CardsVisitor(Cards);

impl<'de> Visitor<'de> for CardsVisitor {
    type Value = Cards;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        write!(formatter, "a sequence of cards")
    }

    fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while let Some(card) = seq.next_element::<Card>()? {
            self.0 |= card;
        }
        Ok(self.0)
    }
}

impl Cards {
    pub const NONE: Cards = Cards {
        bits: 0x0000_0000_0000_0000,
    };
    pub const CHARGEABLE: Cards = Cards {
        bits: 0x0400_1000_0200_0100,
    };
    pub const CLUBS: Cards = Cards {
        bits: 0x0000_0000_0000_1fff,
    };
    pub const DIAMONDS: Cards = Cards {
        bits: 0x0000_0000_1fff_0000,
    };
    pub const HEARTS: Cards = Cards {
        bits: 0x0000_1fff_0000_0000,
    };
    pub const SPADES: Cards = Cards {
        bits: 0x1fff_0000_0000_0000,
    };
    pub const JACK_DIAMONDS: Cards = Cards {
        bits: Self::DIAMONDS.bits & Self::CHARGEABLE.bits,
    };
    pub const QUEEN_SPADES: Cards = Cards {
        bits: Self::SPADES.bits & Self::CHARGEABLE.bits,
    };
    pub const POINTS: Cards = Cards {
        bits: Self::HEARTS.bits | Self::QUEEN_SPADES.bits | Self::JACK_DIAMONDS.bits,
    };
    pub const ALL: Cards = Cards {
        bits: Self::SPADES.bits | Self::HEARTS.bits | Self::DIAMONDS.bits | Self::CLUBS.bits,
    };

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn len(self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn max(self) -> Card {
        Card::from(63 - self.bits.leading_zeros() as u8)
    }

    pub fn min(self) -> Card {
        Card::from(self.bits.trailing_zeros() as u8)
    }

    pub fn contains(self, other: Card) -> bool {
        self == self | other
    }

    pub fn contains_any(self, other: Cards) -> bool {
        self & other != Self::NONE
    }

    pub fn contains_all(self, other: Cards) -> bool {
        self == self | other
    }

    pub fn above(self, card: Card) -> Self {
        Cards {
            bits: (self & card.suit().cards()).bits & !(2 * Cards::from(card).bits - 1),
        }
    }

    pub fn below(self, card: Card) -> Self {
        Cards {
            bits: (self & card.suit().cards()).bits & (Cards::from(card).bits - 1),
        }
    }
}

impl Display for Cards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.into_iter();
        let card = match iter.next() {
            Some(card) => card,
            None => return Ok(()),
        };
        Display::fmt(&card.rank(), f)?;
        let mut prev_suit = card.suit();
        for card in iter {
            if card.suit() != prev_suit {
                Display::fmt(&prev_suit, f)?;
                f.write_char(' ')?;
            }
            Display::fmt(&card.rank(), f)?;
            prev_suit = card.suit();
        }
        Display::fmt(&prev_suit, f)?;
        Ok(())
    }
}

impl Debug for Cards {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl FromStr for Cards {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cards = Cards::NONE;
        let mut chars = s.chars();
        let mut curr_suit = Suit::Clubs;
        while let Some(c) = chars.next_back() {
            if let Ok(rank) = Rank::try_from(c) {
                cards |= Card::new(rank, curr_suit);
            } else if let Ok(suit) = Suit::try_from(c) {
                curr_suit = suit;
            }
        }
        Ok(cards)
    }
}

impl From<Card> for Cards {
    fn from(card: Card) -> Self {
        Cards {
            bits: 1 << (card as u64),
        }
    }
}

impl BitOr<Cards> for Cards {
    type Output = Self;

    fn bitor(self, rhs: Cards) -> Self::Output {
        Cards {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOr<Card> for Cards {
    type Output = Self;

    fn bitor(self, rhs: Card) -> Self::Output {
        self | Self::from(rhs)
    }
}

impl BitOrAssign<Cards> for Cards {
    fn bitor_assign(&mut self, rhs: Cards) {
        self.bits |= rhs.bits;
    }
}

impl BitOrAssign<Card> for Cards {
    fn bitor_assign(&mut self, rhs: Card) {
        *self |= Self::from(rhs)
    }
}

impl BitAnd<Cards> for Cards {
    type Output = Self;

    fn bitand(self, rhs: Cards) -> Self::Output {
        Cards {
            bits: self.bits & rhs.bits,
        }
    }
}

impl BitAnd<Card> for Cards {
    type Output = Self;

    fn bitand(self, rhs: Card) -> Self::Output {
        self & Self::from(rhs)
    }
}

impl BitAndAssign<Cards> for Cards {
    fn bitand_assign(&mut self, rhs: Cards) {
        self.bits &= rhs.bits;
    }
}

impl BitAndAssign<Card> for Cards {
    fn bitand_assign(&mut self, rhs: Card) {
        *self &= Self::from(rhs)
    }
}

impl Sub<Cards> for Cards {
    type Output = Self;

    fn sub(self, rhs: Cards) -> Self::Output {
        Cards {
            bits: self.bits & !rhs.bits,
        }
    }
}

impl Sub<Card> for Cards {
    type Output = Self;

    fn sub(self, rhs: Card) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl SubAssign<Cards> for Cards {
    fn sub_assign(&mut self, rhs: Cards) {
        self.bits &= !rhs.bits;
    }
}

impl SubAssign<Card> for Cards {
    fn sub_assign(&mut self, rhs: Card) {
        *self -= Self::from(rhs)
    }
}

impl IntoIterator for Cards {
    type Item = Card;
    type IntoIter = CardsIter;

    fn into_iter(self) -> Self::IntoIter {
        CardsIter(self)
    }
}

impl IntoIterator for &Cards {
    type Item = Card;
    type IntoIter = CardsIter;

    fn into_iter(self) -> Self::IntoIter {
        CardsIter(*self)
    }
}

impl IntoIterator for &mut Cards {
    type Item = Card;
    type IntoIter = CardsIter;

    fn into_iter(self) -> Self::IntoIter {
        CardsIter(*self)
    }
}

pub struct CardsIter(Cards);

impl Iterator for CardsIter {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == Cards::NONE {
            None
        } else {
            let card = self.0.max();
            self.0 -= card;
            Some(card)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.0.len() as usize;
        (size, Some(size))
    }
}

impl ExactSizeIterator for CardsIter {}

impl FromIterator<Card> for Cards {
    fn from_iter<T: IntoIterator<Item = Card>>(iter: T) -> Self {
        let mut cards = Cards::NONE;
        iter.into_iter().for_each(|c| cards |= c);
        cards
    }
}

impl FromIterator<Cards> for Cards {
    fn from_iter<T: IntoIterator<Item = Cards>>(iter: T) -> Self {
        let mut cards = Cards::NONE;
        iter.into_iter().for_each(|c| cards |= c);
        cards
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(
            format!(
                "{}",
                Card::NineSpades | Card::QueenSpades | Card::JackDiamonds
            ),
            "Q9S JD"
        );
    }

    #[test]
    fn test_max() {
        assert_eq!((Card::TwoClubs | Card::NineClubs).max(), Card::NineClubs);
        assert_eq!(
            (Card::FourHearts | Card::SevenDiamonds).max(),
            Card::FourHearts
        );
        assert_eq!((Card::AceSpades | Card::FiveSpades).max(), Card::AceSpades);
        assert_eq!(Cards::from(Card::FiveHearts).max(), Card::FiveHearts);
    }

    #[test]
    fn test_iter() {
        assert_eq!(
            (Card::QueenSpades | Card::AceHearts | Card::TenClubs | Card::JackDiamonds)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![
                Card::QueenSpades,
                Card::AceHearts,
                Card::JackDiamonds,
                Card::TenClubs
            ]
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(Cards::from(Card::AceHearts), "AH".parse().unwrap())
    }

    #[test]
    fn test_above() {
        assert_eq!(
            Cards::from_str("AQJH").unwrap(),
            Cards::from_str("5S AQJT9H 3C")
                .unwrap()
                .above(Card::TenHearts)
        )
    }

    #[test]
    fn test_below() {
        assert_eq!(
            Cards::from_str("97H").unwrap(),
            Cards::from_str("5S AQJT97H 3C")
                .unwrap()
                .below(Card::TenHearts)
        )
    }
}
