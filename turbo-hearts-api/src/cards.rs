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
    pub const NINES: Cards = Cards {
        bits: 0x0080_0080_0080_0080,
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
        self == Self::NONE
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

    pub fn powerset(self) -> impl Iterator<Item = Cards> {
        Powerset {
            cards: self,
            index: (1 << self.len()) - 1,
        }
    }

    pub fn choose(self, k: usize) -> impl Iterator<Item = Cards> {
        Choose {
            cards: self,
            set: (1 << k) - 1,
        }
    }

    pub fn distinct_plays(self, played: Cards) -> Cards {
        let always_distinct = self & (Cards::NINES | Cards::CHARGEABLE);
        let mut magic = (self - always_distinct).bits;
        let equivalent_blocks = magic | played.bits;
        for _ in 0..11 {
            magic = (magic | (magic >> 1)) & equivalent_blocks;
        }
        magic += ((!magic) << 1) | 0x0001_0001_0001_0001;
        always_distinct
            | Cards {
                bits: self.bits & (magic >> 1),
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

struct Powerset {
    cards: Cards,
    index: usize,
}

impl Iterator for Powerset {
    type Item = Cards;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == usize::MAX {
            return None;
        }
        let mut cards = Cards::NONE;
        let mut index = self.index;
        let mut remaining = self.cards;
        while index != 0 {
            let card = remaining.max();
            if index & 1 == 1 {
                cards |= card;
            }
            index /= 2;
            remaining -= card;
        }
        self.index = self.index.wrapping_sub(1);
        Some(cards)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.index))
    }
}

impl ExactSizeIterator for Powerset {}

struct Choose {
    cards: Cards,
    set: usize,
}

impl Iterator for Choose {
    type Item = Cards;

    fn next(&mut self) -> Option<Self::Item> {
        if self.set >= 1 << self.cards.len() {
            return None;
        }
        let mut cards = Cards::NONE;
        let mut set = self.set;
        let mut remaining = self.cards;
        while set != 0 {
            let card = remaining.max();
            if set & 1 == 1 {
                cards |= card;
            }
            set /= 2;
            remaining -= card;
        }
        let c = self.set & self.set.wrapping_neg();
        let r = self.set + c;
        self.set = (((r ^ self.set) >> 2) / c) | r;
        Some(cards)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! c {
        ($($cards:tt)*) => {
            stringify!($($cards)*).parse::<Cards>().unwrap()
        };
    }

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
        assert_eq!(c!(AQJH), c!(5S AQJT9H 3C).above(Card::TenHearts))
    }

    #[test]
    fn test_below() {
        assert_eq!(c!(97H), c!(5S AQJT97H 3C).below(Card::TenHearts))
    }

    #[test]
    fn test_powerset_none() {
        let mut pset = Cards::NONE.powerset();
        assert_eq!(pset.next(), Some(Cards::NONE));
        assert_eq!(pset.next(), None);
    }

    #[test]
    fn test_powerset_one() {
        let mut pset = Cards::from(Card::FiveSpades).powerset();
        assert_eq!(pset.next(), Some(Cards::from(Card::FiveSpades)));
        assert_eq!(pset.next(), Some(Cards::NONE));
        assert_eq!(pset.next(), None);
    }

    #[test]
    fn test_powerset_two() {
        let mut pset = c!(QS TC).powerset();
        assert_eq!(pset.next(), Some(c!(QS TC)));
        assert_eq!(pset.next(), Some(c!(TC)));
        assert_eq!(pset.next(), Some(c!(QS)));
        assert_eq!(pset.next(), Some(Cards::NONE));
        assert_eq!(pset.next(), None);
    }

    #[test]
    fn test_powerset_three() {
        let mut pset = c!(QS AH TC).powerset();
        assert_eq!(pset.next(), Some(c!(QS AH TC)));
        assert_eq!(pset.next(), Some(c!(AH TC)));
        assert_eq!(pset.next(), Some(c!(QS TC)));
        assert_eq!(pset.next(), Some(c!(TC)));
        assert_eq!(pset.next(), Some(c!(QS AH)));
        assert_eq!(pset.next(), Some(c!(AH)));
        assert_eq!(pset.next(), Some(c!(QS)));
        assert_eq!(pset.next(), Some(Cards::NONE));
        assert_eq!(pset.next(), None);
    }

    #[test]
    fn test_choose() {
        let mut choose = c!(AKQJTS).choose(3);
        assert_eq!(choose.next(), Some(c!(AKQS)));
        assert_eq!(choose.next(), Some(c!(AKJS)));
        assert_eq!(choose.next(), Some(c!(AQJS)));
        assert_eq!(choose.next(), Some(c!(KQJS)));
        assert_eq!(choose.next(), Some(c!(AKTS)));
        assert_eq!(choose.next(), Some(c!(AQTS)));
        assert_eq!(choose.next(), Some(c!(KQTS)));
        assert_eq!(choose.next(), Some(c!(AJTS)));
        assert_eq!(choose.next(), Some(c!(KJTS)));
        assert_eq!(choose.next(), Some(c!(QJTS)));
        assert_eq!(choose.next(), None);
    }

    #[test]
    fn test_distinct_plays() {
        assert_eq!(c!(24C).distinct_plays(c!(5C)), c!(24C));
        assert_eq!(c!(24C).distinct_plays(c!(3C)), c!(4C));
        assert_eq!(c!(2H AD).distinct_plays(c!(3H KD)), c!(2H AD));
        assert_eq!(c!(T9S).distinct_plays(c!(3H KD)), c!(T9S));
        assert_eq!(c!(JT9S).distinct_plays(c!(3H KD)), c!(J9S));
        assert_eq!(c!(J9S).distinct_plays(c!(TS)), c!(J9S));
        assert_eq!(c!(QJT9S).distinct_plays(c!(3S)), c!(QJ9S));
        assert_eq!(c!(KJ7532S).distinct_plays(c!(A6S)), c!(KJ73S));
        assert_eq!(c!(QJT987C).distinct_plays(c!(A6S)), c!(QT98C));
        assert_eq!(c!(KQJT987D).distinct_plays(c!(A6S)), c!(KJT98D));
    }
}
