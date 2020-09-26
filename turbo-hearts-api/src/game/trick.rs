use crate::{Card, Cards, Rank, Seat, Suit};

#[derive(Clone, Debug)]
pub struct Trick {
    cards: [Card; 8],
    len: u8,
}

impl Trick {
    pub fn new() -> Self {
        Self {
            cards: [Card::TwoClubs; 8],
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn suit(&self) -> Suit {
        self.cards[0].suit()
    }

    pub fn is_complete(&self) -> bool {
        self.len == 8 || (self.len == 4 && !self.nined())
    }

    pub fn cards(&self) -> Cards {
        self.slice().iter().cloned().collect()
    }

    pub fn winning_card(&self) -> Card {
        let suit = self.suit();
        self.slice()
            .iter()
            .cloned()
            .filter(|c| c.suit() == suit)
            .max()
            .unwrap()
    }

    pub fn winning_seat(&self, next: Seat) -> Seat {
        let cards = self.slice();
        let mut index = 0;
        let mut max = cards[0];
        for i in 1..cards.len() {
            let card = cards[i];
            if card.suit() == max.suit() && card > max {
                max = card;
                index = i;
            }
        }
        match (self.len - index as u8) % 4 {
            0 => next,
            1 => next.right(),
            2 => next.across(),
            _ => next.left(),
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards[self.len as usize] = card;
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    fn nined(&self) -> bool {
        let nine = self.suit().with_rank(Rank::Nine);
        self.slice().contains(&nine)
    }

    fn slice(&self) -> &[Card] {
        &self.cards[..self.len as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_empty() {
        let mut trick = Trick::new();
        assert!(trick.is_empty());
        trick.push(Card::FiveClubs);
        assert!(!trick.is_empty());
    }

    #[test]
    fn test_suit() {
        let mut trick = Trick::new();
        trick.push(Card::FiveClubs);
        assert_eq!(trick.suit(), Suit::Clubs);
    }

    #[test]
    fn test_nined() {
        let mut trick = Trick::new();
        assert!(!trick.nined());
        trick.push(Card::FiveClubs);
        assert!(!trick.nined());
        trick.push(Card::NineDiamonds);
        assert!(!trick.nined());
        trick.push(Card::NineClubs);
        assert!(trick.nined());
        trick.push(Card::FourClubs);
        assert!(trick.nined());
    }

    #[test]
    fn test_is_complete() {
        let mut trick = Trick::new();
        assert!(!trick.is_complete());
        trick.push(Card::FiveClubs);
        assert!(!trick.is_complete());
        trick.push(Card::NineDiamonds);
        assert!(!trick.is_complete());
        trick.push(Card::TenClubs);
        assert!(!trick.is_complete());
        trick.push(Card::FourClubs);
        assert!(trick.is_complete());
    }

    #[test]
    fn test_is_complete_nined() {
        let mut trick = Trick::new();
        assert!(!trick.is_complete());
        trick.push(Card::FiveClubs);
        assert!(!trick.is_complete());
        trick.push(Card::NineDiamonds);
        assert!(!trick.is_complete());
        trick.push(Card::NineClubs);
        assert!(!trick.is_complete());
        trick.push(Card::FourClubs);
        assert!(!trick.is_complete());
        trick.push(Card::AceClubs);
        assert!(!trick.is_complete());
        trick.push(Card::EightHearts);
        assert!(!trick.is_complete());
        trick.push(Card::TwoSpades);
        assert!(!trick.is_complete());
        trick.push(Card::KingClubs);
        assert!(trick.is_complete());
    }

    #[test]
    fn test_cards() {
        let mut trick = Trick::new();
        trick.push(Card::FiveClubs);
        trick.push(Card::NineDiamonds);
        trick.push(Card::NineClubs);
        trick.push(Card::FourClubs);
        trick.push(Card::AceClubs);
        trick.push(Card::EightHearts);
        assert_eq!(trick.cards(), "8H 9D A954C".parse().unwrap());
    }

    #[test]
    fn test_winning_card() {
        let mut trick = Trick::new();
        trick.push(Card::FiveClubs);
        trick.push(Card::NineDiamonds);
        trick.push(Card::NineClubs);
        trick.push(Card::FourClubs);
        trick.push(Card::KingClubs);
        trick.push(Card::AceHearts);
        assert_eq!(trick.winning_card(), Card::KingClubs);
    }

    #[test]
    fn test_winning_seat() {
        let mut trick = Trick::new();
        trick.push(Card::FiveClubs);
        assert_eq!(trick.winning_seat(Seat::West), Seat::South);
        trick.push(Card::NineDiamonds);
        assert_eq!(trick.winning_seat(Seat::North), Seat::South);
        trick.push(Card::NineClubs);
        assert_eq!(trick.winning_seat(Seat::East), Seat::North);
        trick.push(Card::FourClubs);
        assert_eq!(trick.winning_seat(Seat::South), Seat::North);
        trick.push(Card::KingClubs);
        assert_eq!(trick.winning_seat(Seat::West), Seat::South);
        trick.push(Card::AceHearts);
        assert_eq!(trick.winning_seat(Seat::North), Seat::South);
    }
}
