use crate::{Card, Cards, Seat};

pub struct WonState {
    state: u32,
}

impl WonState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn win(&mut self, seat: Seat, cards: Cards) {
        let mut update = 0;
        update += (cards & Cards::HEARTS).len();
        if cards.contains(Card::QueenSpades) {
            update += 16;
        }
        if cards.contains(Card::JackDiamonds) {
            update += 32;
        }
        if cards.contains(Card::TenClubs) {
            update += 64;
        }
        self.state += update << (8 * seat.idx());
    }

    pub fn hearts_broken(&self) -> bool {
        self.state & 0x0f_0f_0f_0f != 0
    }

    pub fn can_run(&self, seat: Seat) -> bool {
        self.state & (0x1f_1f_1f_1f ^ (0x1f << (8 * seat.idx()))) == 0
    }

    pub fn hearts(&self, seat: Seat) -> u8 {
        ((self.state >> (8 * seat.idx())) & 0xf) as u8
    }

    pub fn queen(&self, seat: Seat) -> bool {
        self.state & (0x10 << (8 * seat.idx())) != 0
    }

    pub fn queen_winner(&self) -> Option<Seat> {
        Seat::VALUES
            .get((self.state & 0x10_10_10_10).trailing_zeros() / 8)
            .cloned()
    }

    pub fn jack(&self, seat: Seat) -> bool {
        self.state & (0x20 << (8 * seat.idx())) != 0
    }

    pub fn jack_winner(&self) -> Option<Seat> {
        Seat::VALUES
            .get((self.state & 0x20_20_20_20).trailing_zeros() / 8)
            .cloned()
    }

    pub fn ten(&self, seat: Seat) -> bool {
        self.state & (0x40 << (8 * seat.idx())) != 0
    }

    pub fn ten_winner(&self) -> Option<Seat> {
        Seat::VALUES
            .get((self.state & 0x40_40_40_40).trailing_zeros() / 8)
            .cloned()
    }

    pub fn score(&self, seat: Seat, charged: Cards) -> i16 {
        let hearts = if charged.contains(Card::AceHearts) {
            2 * self.hearts(seat)
        } else {
            self.hearts(seat)
        } as i16;

        let queen = match (self.queen(seat), charged.contains(Card::QueenSpades)) {
            (true, true) => 26,
            (true, false) => 13,
            _ => 0,
        };
        let jack = match (self.jack(seat), charged.contains(Card::JackDiamonds)) {
            (true, true) => -20,
            (true, false) => -10,
            _ => 0,
        };
        let ten = match (self.ten(seat), charged.contains(Card::TenClubs)) {
            (true, true) => 4,
            (true, false) => 2,
            _ => 1,
        };
        if self.queen(seat) && self.hearts(seat) == 13 {
            ten * (jack - hearts - queen)
        } else {
            ten * (jack + hearts + queen)
        }
    }
}
