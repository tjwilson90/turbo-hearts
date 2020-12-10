use crate::{Card, Cards, Seat, Suit};
use std::{fmt, fmt::Formatter};

const NINE_MASKS: [u32; 4] = [0x07_07_07_07, 0x17_17_17_17, 0x27_27_27_27, 0x37_37_37_37];
const SUIT_XORS: [u64; 4] = [
    0xB0_B0_B0_B0_B0_B0_B0_B0,
    0xA0_A0_A0_A0_A0_A0_A0_A0,
    0x90_90_90_90_90_90_90_90,
    0x80_80_80_80_80_80_80_80,
];
const EMPTY: u64 = 0x80_80_80_80_80_80_80_80;

#[derive(Clone, Copy)]
pub struct Trick {
    state: u64,
}

impl Trick {
    pub fn new() -> Self {
        Self { state: EMPTY }
    }

    pub fn is_empty(self) -> bool {
        self.state == EMPTY
    }

    pub fn len(self) -> usize {
        (8 - (self.state ^ EMPTY).leading_zeros() / 8) as usize
    }

    pub fn suit(self) -> Suit {
        let shift = 60 - (self.state ^ EMPTY).leading_zeros();
        Suit::from(((self.state >> shift) & 3) as u8)
    }

    pub fn is_complete(self) -> bool {
        if self.state & EMPTY == 0 {
            return true;
        }
        if self.state & EMPTY != 0x80_80_80_80_00_00_00_00 {
            return false;
        }
        let suit = (self.state >> 28) & 3;
        let xor = self.state as u32 ^ NINE_MASKS[suit as usize];
        xor.wrapping_sub(0x01_01_01_01) & (!xor) & 0x80_80_80_80 == 0
    }

    pub fn cards(self) -> Cards {
        let mut cards = Cards::NONE;
        let mut state = self.state;
        for _ in 0..8 - (self.state ^ EMPTY).leading_zeros() / 8 {
            cards |= Card::from(state as u8);
            state >>= 8;
        }
        cards
    }

    pub fn winning_seat(self, next: Seat) -> Seat {
        let leading = (self.state ^ EMPTY).leading_zeros();
        let first_index = 7 - leading / 8;
        let suit = (self.state >> (60 - leading)) & 3;
        let mut state = self.state ^ SUIT_XORS[suit as usize];
        let mut max = (state >> (56 - leading)) as u8;
        let mut index = first_index;
        for i in 0..first_index {
            let byte = state as u8;
            if byte > max {
                max = byte;
                index = i;
            }
            state >>= 8;
        }
        match index % 4 {
            0 => next.right(),
            1 => next.across(),
            2 => next.left(),
            _ => next,
        }
    }

    #[must_use]
    pub fn push(self, card: Card) -> Trick {
        Self {
            state: (self.state << 8) | (card as u8 as u64),
        }
    }
}

impl fmt::Debug for Trick {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let mut list = f.debug_list();
        for card in *self {
            list.entry(&card);
        }
        list.finish()
    }
}

impl IntoIterator for Trick {
    type Item = Card;
    type IntoIter = TrickIter;

    fn into_iter(self) -> Self::IntoIter {
        TrickIter(self.state)
    }
}

pub struct TrickIter(u64);

impl Iterator for TrickIter {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == EMPTY {
            return None;
        }
        let card = Card::from((self.0 & 0xff) as u8);
        self.0 >>= 8;
        self.0 |= 0x80_00_00_00_00_00_00_00;
        Some(card)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = Trick { state: self.0 }.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for TrickIter {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_empty() {
        let trick = Trick::new();
        assert!(trick.is_empty());
        assert!(!trick.push(Card::FiveClubs).is_empty());
    }

    #[test]
    fn test_suit() {
        let mut trick = Trick::new().push(Card::FiveClubs);
        assert_eq!(trick.suit(), Suit::Clubs);
        trick = trick.push(Card::ThreeHearts);
        assert_eq!(trick.suit(), Suit::Clubs);
    }

    #[test]
    fn test_is_complete() {
        let mut trick = Trick::new();
        assert!(!trick.is_complete());
        trick = trick.push(Card::FiveClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::NineDiamonds);
        assert!(!trick.is_complete());
        trick = trick.push(Card::TenClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::FourClubs);
        assert!(trick.is_complete());
    }

    #[test]
    fn test_is_complete_nined() {
        let mut trick = Trick::new();
        assert!(!trick.is_complete());
        trick = trick.push(Card::FiveClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::NineDiamonds);
        assert!(!trick.is_complete());
        trick = trick.push(Card::NineClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::FourClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::AceClubs);
        assert!(!trick.is_complete());
        trick = trick.push(Card::EightHearts);
        assert!(!trick.is_complete());
        trick = trick.push(Card::TwoSpades);
        assert!(!trick.is_complete());
        trick = trick.push(Card::KingClubs);
        assert!(trick.is_complete());
    }

    #[test]
    fn test_cards() {
        let mut trick = Trick::new().push(Card::FiveClubs);
        assert_eq!(trick.cards(), "5C".parse().unwrap());
        trick = trick.push(Card::NineDiamonds);
        assert_eq!(trick.cards(), "9D 5C".parse().unwrap());
        trick = trick.push(Card::NineClubs);
        assert_eq!(trick.cards(), "9D 95C".parse().unwrap());
        trick = trick.push(Card::FourClubs);
        assert_eq!(trick.cards(), "9D 954C".parse().unwrap());
        trick = trick.push(Card::AceClubs);
        assert_eq!(trick.cards(), "9D A954C".parse().unwrap());
        trick = trick.push(Card::EightHearts);
        assert_eq!(trick.cards(), "8H 9D A954C".parse().unwrap());
    }

    #[test]
    fn test_winning_seat() {
        let mut trick = Trick::new().push(Card::FiveClubs);
        assert_eq!(trick.winning_seat(Seat::West), Seat::South);
        trick = trick.push(Card::NineDiamonds);
        assert_eq!(trick.winning_seat(Seat::North), Seat::South);
        trick = trick.push(Card::NineClubs);
        assert_eq!(trick.winning_seat(Seat::East), Seat::North);
        trick = trick.push(Card::FourClubs);
        assert_eq!(trick.winning_seat(Seat::South), Seat::North);
        trick = trick.push(Card::KingClubs);
        assert_eq!(trick.winning_seat(Seat::West), Seat::South);
        trick = trick.push(Card::AceHearts);
        assert_eq!(trick.winning_seat(Seat::North), Seat::South);
    }

    #[test]
    fn test_into_iter() {
        let trick = Trick::new()
            .push(Card::FiveClubs)
            .push(Card::NineDiamonds)
            .push(Card::NineClubs);
        let mut iter = trick.into_iter();
        assert_eq!(iter.next(), Some(Card::NineClubs));
        assert_eq!(iter.next(), Some(Card::NineDiamonds));
        assert_eq!(iter.next(), Some(Card::FiveClubs));
        assert_eq!(iter.next(), None);
    }
}
