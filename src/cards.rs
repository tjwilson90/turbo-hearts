use serde::{Deserialize, Serialize};
use std::{
    convert::{Infallible, TryFrom},
    fmt,
    fmt::{Debug, Display, Write},
    iter::FromIterator,
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign},
    str::FromStr,
};

const RANKS: [char; 13] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Rank {
    Two = 0,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn char(self) -> char {
        RANKS[self as usize]
    }
}

impl From<u8> for Rank {
    fn from(n: u8) -> Self {
        assert!(n < 13, "n={}", n);
        unsafe { mem::transmute(n) }
    }
}

impl TryFrom<char> for Rank {
    type Error = char;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        RANKS
            .iter()
            .position(|&r| r == c)
            .map(|n| Self::from(n as u8))
            .ok_or(c)
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.char())
    }
}

impl Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

const SUITS: [char; 4] = ['C', 'D', 'H', 'S'];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Suit {
    Clubs = 0,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub fn char(self) -> char {
        SUITS[self as usize]
    }

    pub fn cards(self) -> Cards {
        Cards {
            bits: 0x1fff << (16 * self as u64),
        }
    }

    pub fn nine(self) -> Card {
        Card::new(Rank::Nine, self)
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

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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

    pub fn suit(self) -> Suit {
        Suit::from(self as u8 / 16)
    }

    pub fn parse(s: &str) -> Self {
        s.parse().unwrap()
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(from = "String")]
#[serde(into = "String")]
pub struct Cards {
    pub bits: u64,
}

impl Cards {
    pub const NONE: Cards = Cards {
        bits: 0x0000_0000_0000_0000,
    };
    pub const CHARGEABLE: Cards = Cards {
        bits: 0x0400_1000_0200_0100,
    };
    pub const NINES: Cards = Cards {
        bits: 0x0080_0080_0080_0080,
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
    pub const TWO_CLUBS: Cards = Cards {
        bits: 0x0000_0000_0000_0001,
    };
    pub const TEN_CLUBS: Cards = Cards {
        bits: Self::CLUBS.bits & Self::CHARGEABLE.bits,
    };
    pub const JACK_DIAMONDS: Cards = Cards {
        bits: Self::DIAMONDS.bits & Self::CHARGEABLE.bits,
    };
    pub const ACE_HEARTS: Cards = Cards {
        bits: Self::HEARTS.bits & Self::CHARGEABLE.bits,
    };
    pub const QUEEN_SPADES: Cards = Cards {
        bits: Self::SPADES.bits & Self::CHARGEABLE.bits,
    };
    pub const RUNNING: Cards = Cards {
        bits: Self::HEARTS.bits | Self::QUEEN_SPADES.bits,
    };
    pub const POINTS: Cards = Cards {
        bits: Self::RUNNING.bits | Self::JACK_DIAMONDS.bits,
    };
    pub const ALL: Cards = Cards {
        bits: Self::SPADES.bits | Self::HEARTS.bits | Self::DIAMONDS.bits | Self::CLUBS.bits,
    };

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn len(self) -> u32 {
        self.bits.count_ones()
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

    pub fn suit(self) -> Self {
        if self == Self::NONE {
            Self::NONE
        } else {
            self.max().suit().cards()
        }
    }

    pub fn parse(s: &str) -> Self {
        s.parse().unwrap()
    }
}

impl fmt::Display for Cards {
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

impl From<String> for Cards {
    fn from(s: String) -> Self {
        Cards::from_str(&s).unwrap()
    }
}

impl From<Cards> for String {
    fn from(c: Cards) -> Self {
        c.to_string()
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

impl BitXor<Cards> for Cards {
    type Output = Self;

    fn bitxor(self, rhs: Cards) -> Self::Output {
        Cards {
            bits: self.bits ^ rhs.bits,
        }
    }
}

impl BitXor<Card> for Cards {
    type Output = Self;

    fn bitxor(self, rhs: Card) -> Self::Output {
        self ^ Self::from(rhs)
    }
}

impl BitXorAssign<Cards> for Cards {
    fn bitxor_assign(&mut self, rhs: Cards) {
        self.bits ^= rhs.bits;
    }
}

impl BitXorAssign<Card> for Cards {
    fn bitxor_assign(&mut self, rhs: Card) {
        *self ^= Self::from(rhs)
    }
}

impl Not for Cards {
    type Output = Self;

    fn not(self) -> Self::Output {
        Cards {
            bits: Self::ALL.bits - self.bits,
        }
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

impl DoubleEndedIterator for CardsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0 == Cards::NONE {
            None
        } else {
            let card = self.0.min();
            self.0 -= card;
            Some(card)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_display() {
        assert_eq!(format!("{}", Card::NineSpades), "9S");
        assert_eq!(format!("{}", Card::ThreeDiamonds), "3D");
        assert_eq!(format!("{}", Card::JackClubs), "JC");
        assert_eq!(format!("{}", Card::AceHearts), "AH");
    }

    #[test]
    fn test_card_parse() {
        assert_eq!(Card::parse("TH"), Card::TenHearts);
        assert_eq!(Card::parse("3D"), Card::ThreeDiamonds);
        assert_eq!(Card::parse("4C"), Card::FourClubs);
        assert_eq!(Card::parse("AS"), Card::AceSpades);
    }

    #[test]
    fn test_card_suit() {
        assert_eq!(Card::TwoClubs.suit().cards(), Cards::CLUBS);
        assert_eq!(Card::AceClubs.suit().cards(), Cards::CLUBS);
        assert_eq!(Card::TwoDiamonds.suit().cards(), Cards::DIAMONDS);
        assert_eq!(Card::AceDiamonds.suit().cards(), Cards::DIAMONDS);
        assert_eq!(Card::TwoHearts.suit().cards(), Cards::HEARTS);
        assert_eq!(Card::AceHearts.suit().cards(), Cards::HEARTS);
        assert_eq!(Card::TwoSpades.suit().cards(), Cards::SPADES);
        assert_eq!(Card::AceSpades.suit().cards(), Cards::SPADES);
    }

    #[test]
    fn test_cards_display() {
        assert_eq!(
            format!(
                "{}",
                Card::NineSpades | Card::QueenSpades | Card::JackDiamonds
            ),
            "Q9S JD"
        );
    }

    #[test]
    fn test_cards_max() {
        assert_eq!(Cards::parse("98765432C").max(), Card::NineClubs);
        assert_eq!(Cards::parse("98765D 432H").max(), Card::FourHearts);
        assert_eq!(Cards::parse("A5S").max(), Card::AceSpades);
        assert_eq!(Cards::parse("5H").max(), Card::FiveHearts);
    }

    #[test]
    fn test_cards_min() {
        assert_eq!(Cards::parse("98765432C").min(), Card::TwoClubs);
        assert_eq!(Cards::parse("98765D 432H").min(), Card::FiveDiamonds);
        assert_eq!(Cards::parse("A5S").min(), Card::FiveSpades);
        assert_eq!(Cards::parse("5H").min(), Card::FiveHearts);
    }

    #[test]
    fn test_cards_parse() {
        assert_eq!(
            Cards::parse("QS AH JD AKQJT98765432C"),
            Card::QueenSpades | Card::AceHearts | Card::JackDiamonds | Cards::CLUBS
        );
    }

    #[test]
    fn test_cards_iter() {
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
}
