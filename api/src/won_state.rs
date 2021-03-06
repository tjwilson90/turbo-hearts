use crate::{Card, Cards, ChargeState, Scores, Seat};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WonState {
    state: u32,
}

impl WonState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub fn win(self, seat: Seat, cards: Cards) -> Self {
        let mut update = 0;
        update += (cards & Cards::HEARTS).len() as u32;
        if cards.contains(Card::QueenSpades) {
            update += 16;
        }
        if cards.contains(Card::JackDiamonds) {
            update += 32;
        }
        if cards.contains(Card::TenClubs) {
            update += 64;
        }
        Self {
            state: self.state + (update << (8 * seat.idx())),
        }
    }

    #[must_use]
    pub fn claim(self, seat: Seat) -> Self {
        let mut update = 29;
        if self.state & 0x20_20_20_20 == 0 {
            update += 32;
        }
        if self.state & 0x40_40_40_40 == 0 {
            update += 64;
        }
        Self {
            state: (self.state & 0xf0_f0_f0_f0) | (update << (8 * seat.idx())),
        }
    }

    pub fn hearts_broken(self) -> bool {
        self.state & 0x0f_0f_0f_0f != 0
    }

    pub fn can_run(self, seat: Seat) -> bool {
        self.state & (0x1f_1f_1f_1f ^ (0x1f << (8 * seat.idx()))) == 0
    }

    pub fn hearts(self, seat: Seat) -> u8 {
        ((self.state >> (8 * seat.idx())) & 0xf) as u8
    }

    pub fn queen(self, seat: Seat) -> bool {
        self.state & (0x10 << (8 * seat.idx())) != 0
    }

    pub fn queen_winner(self) -> Option<Seat> {
        Seat::VALUES
            .get(((self.state & 0x10_10_10_10).trailing_zeros() / 8) as usize)
            .cloned()
    }

    pub fn jack(self, seat: Seat) -> bool {
        self.state & (0x20 << (8 * seat.idx())) != 0
    }

    pub fn jack_winner(self) -> Option<Seat> {
        Seat::VALUES
            .get(((self.state & 0x20_20_20_20).trailing_zeros() / 8) as usize)
            .cloned()
    }

    pub fn ten(self, seat: Seat) -> bool {
        self.state & (0x40 << (8 * seat.idx())) != 0
    }

    pub fn ten_winner(self) -> Option<Seat> {
        Seat::VALUES
            .get(((self.state & 0x40_40_40_40).trailing_zeros() / 8) as usize)
            .cloned()
    }

    pub fn scores(self, charges: ChargeState) -> Scores {
        let heart_multiplier = if charges.is_charged(Card::AceHearts) {
            2
        } else {
            1
        };
        let mut scores = [
            heart_multiplier * self.hearts(Seat::North) as i16,
            heart_multiplier * self.hearts(Seat::East) as i16,
            heart_multiplier * self.hearts(Seat::South) as i16,
            heart_multiplier * self.hearts(Seat::West) as i16,
        ];
        self.queen_winner().map(|s| {
            scores[s.idx()] += if charges.is_charged(Card::QueenSpades) {
                26
            } else {
                13
            };
            if self.hearts(s) == 13 {
                scores[s.idx()] *= -1;
            }
        });
        self.jack_winner().map(|s| {
            scores[s.idx()] += if charges.is_charged(Card::JackDiamonds) {
                -20
            } else {
                -10
            };
        });
        self.ten_winner().map(|s| {
            scores[s.idx()] *= if charges.is_charged(Card::TenClubs) {
                4
            } else {
                2
            };
        });
        Scores::new(scores)
    }
}

impl fmt::Debug for WonState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        for &seat in &Seat::VALUES {
            if seat != Seat::North {
                write!(f, ", ")?;
            }
            write!(f, "{} [{}H", seat, self.hearts(seat))?;
            if self.queen(seat) {
                write!(f, ", QS")?;
            }
            if self.jack(seat) {
                write!(f, ", JD")?;
            }
            if self.ten(seat) {
                write!(f, ", TC")?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_claim() {
        let claim = WonState::new()
            .win(Seat::North, "AD 364H".parse().unwrap())
            .win(Seat::East, "T876C".parse().unwrap())
            .claim(Seat::North);
        assert_eq!(
            claim,
            WonState::new()
                .win(Seat::North, "QS AKQJT98765432H JD".parse().unwrap())
                .win(Seat::East, "TC".parse().unwrap())
        )
    }
}
