use crate::types::{ChargingRules, PassDirection, Seat};
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
    mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign},
    str::FromStr,
};

const RANKS: [char; 13] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
];

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Rank {
    Two,
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
    Clubs,
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

    pub fn contains(self, other: Card) -> bool {
        self == self | other
    }

    pub fn contains_any(self, other: Cards) -> bool {
        self & other != Self::NONE
    }

    pub fn contains_all(self, other: Cards) -> bool {
        self == self | other
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

#[derive(Debug, Clone)]
pub struct HandState {
    pub played: Cards,
    pub led_suits: Cards,
    pub trick_number: usize,
    pub next_player: Seat,
    pub previous_trick: Vec<Card>,
    pub current_trick: Vec<Card>,
}

impl HandState {
    pub fn new(first_player: Seat) -> Self {
        Self {
            played: Cards::NONE,
            led_suits: Cards::NONE,
            trick_number: 0,
            next_player: first_player,
            previous_trick: Vec::with_capacity(8),
            current_trick: Vec::with_capacity(8),
        }
    }

    pub fn reset(&mut self, first_player: Seat) {
        self.played = Cards::NONE;
        self.led_suits = Cards::NONE;
        self.trick_number = 0;
        self.next_player = first_player;
        self.previous_trick.clear();
        self.current_trick.clear();
    }

    pub fn play(&mut self, card: Card) {
        self.played |= card;
        if self.current_trick.is_empty() {
            self.led_suits |= card.suit().cards();
        }
        self.current_trick.push(card);
        self.next_player = self.next_player.left();
        if self.current_trick.len() == 8
            || self.played == Cards::ALL
            || (self.current_trick.len() == 4
                && !self
                    .current_trick
                    .contains(&self.current_trick[0].suit().nine()))
        {
            let mut seat = self.next_player;
            let mut winning_seat = self.next_player;
            let mut winning_card = self.current_trick[0];
            for card in &self.current_trick[1..] {
                seat = seat.left();
                if card.suit() == winning_card.suit() && card.rank() > winning_card.rank() {
                    winning_card = *card;
                    winning_seat = seat;
                }
            }
            self.next_player = winning_seat;
            mem::swap(&mut self.current_trick, &mut self.previous_trick);
            self.current_trick.clear();
            self.trick_number += 1;
        }
    }
}

#[derive(Debug)]
pub struct ChargeState {
    pub rules: ChargingRules,
    pub pass_direction: PassDirection,
    pub charged: Cards,
    pub done_charging: [bool; 4],
    pub next_charger: Option<Seat>,
}

impl ChargeState {
    pub fn new(rules: ChargingRules, pass_direction: PassDirection) -> Self {
        Self {
            rules,
            pass_direction,
            charged: Cards::NONE,
            done_charging: [false, false, false, false],
            next_charger: if rules.free() {
                None
            } else {
                Some(pass_direction.first_charger())
            },
        }
    }

    pub fn reset_for_round(&mut self, pass_direction: PassDirection) {
        self.pass_direction = pass_direction;
        if pass_direction != PassDirection::Keeper {
            self.charged = Cards::NONE;
        }
        self.done_charging = [false, false, false, false];
        self.next_charger = if self.rules.free() {
            None
        } else {
            Some(pass_direction.first_charger())
        }
    }

    pub fn reset_for_first_keeper(&mut self) {
        self.pass_direction = PassDirection::Keeper;
        self.charged = Cards::NONE;
        self.done_charging = [false, false, false, false];
        self.next_charger = if self.rules.free() {
            None
        } else {
            Some(self.pass_direction.first_charger())
        }
    }

    pub fn can_charge(&self, seat: Seat) -> bool {
        self.next_charger.map_or(true, |s| s == seat)
    }

    pub fn done_charging(&self, seat: Seat) -> bool {
        self.done_charging[seat.idx()]
    }

    pub fn all_done_charging(&self) -> bool {
        self.done_charging.iter().all(|done| *done)
    }

    pub fn charge(&mut self, seat: Seat, cards: Cards) -> bool {
        self.charged |= cards;
        self.blind_charge(seat, cards.len())
    }

    pub fn blind_charge(&mut self, seat: Seat, count: usize) -> bool {
        if let Some(charger) = &mut self.next_charger {
            *charger = charger.left();
        }
        if count == 0 {
            self.done_charging[seat.idx()] = true;
        } else {
            for done_charging in &mut self.done_charging {
                *done_charging = false;
            }
            self.done_charging[seat.idx()] = !self.rules.chain();
        }
        self.all_done_charging()
    }
}

pub fn legal_plays(cards: Cards, hand: &HandState, charged: Cards) -> Cards {
    let mut plays = cards - hand.played;
    // if this is the first trick
    if hand.led_suits.is_empty() {
        // if you have the two of clubs
        if plays.contains(Card::TwoClubs) {
            // you must play it
            return Card::TwoClubs.into();
        }

        // if you have a non-point card
        if !Cards::POINTS.contains_all(plays) {
            // you cannot play points
            plays -= Cards::POINTS;

        // otherwise, if you have the jack of diamonds
        } else if plays.contains(Card::JackDiamonds) {
            // you must play it
            return Card::JackDiamonds.into();

        // otherwise, if you have the queen of spades
        } else if plays.contains(Card::QueenSpades) {
            // you must play it
            return Card::QueenSpades.into();
        }
    }

    // if this is not the first card in the trick
    if !hand.current_trick.is_empty() {
        let suit = hand.current_trick[0].suit();

        // and you have any cards in suit
        if suit.cards().contains_any(plays) {
            // you must play in suit
            plays &= suit.cards();

            // and if this is the first trick of this suit
            if !hand.led_suits.contains_any(suit.cards())
                // and you have multiple plays
                && plays.len() > 1
            {
                // you cannot play charged cards from the suit
                plays -= charged;
            }
        }

    // otherwise, you are leading the trick
    } else {
        // If hearts are not broken
        if !hand.played.contains_any(Cards::HEARTS)
            // and you have a non-heart
            && !Cards::HEARTS.contains_all(plays)
        {
            // you cannot lead hearts
            plays -= Cards::HEARTS;
        }

        let unled_charges = charged - hand.led_suits;
        // if you have cards other than charged cards from unled suits
        if !unled_charges.contains_all(plays) {
            // you must lead one of them
            plays -= unled_charges;
        }
    }
    plays
}
