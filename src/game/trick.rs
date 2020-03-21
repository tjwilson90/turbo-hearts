use crate::{card::Card, cards::Cards, rank::Rank, seat::Seat, suit::Suit};

#[derive(Debug)]
pub struct Trick {
    cards: [Card; 8],
    len: usize,
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

    pub fn nined(&self) -> bool {
        let nine = self.suit().with_rank(Rank::Nine);
        self.cards[..self.len].contains(&nine)
    }

    pub fn is_complete(&self) -> bool {
        self.len == 8 || (self.len == 4 && !self.nined())
    }

    pub fn cards(&self) -> Cards {
        self.cards[..self.len].iter().cloned().collect()
    }

    pub fn winning_card(&self) -> Card {
        let suit = self.suit();
        self.cards[..self.len]
            .iter()
            .cloned()
            .filter(|c| c.suit() == suit)
            .max()
            .unwrap()
    }

    pub fn winning_seat(&self, next: Seat) -> Seat {
        let suit = self.suit();
        let (index, _) = self.cards[..self.len]
            .iter()
            .cloned()
            .enumerate()
            .filter(|(_, c)| c.suit() == suit)
            .max()
            .unwrap();
        match (self.len - index) % 4 {
            0 => next,
            1 => next.right(),
            2 => next.across(),
            _ => next.left(),
        }
    }

    pub fn push(&mut self, card: Card) {
        self.cards[self.len] = card;
        self.len += 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}
