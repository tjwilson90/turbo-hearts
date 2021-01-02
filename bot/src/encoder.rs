use std::collections::HashMap;
use turbo_hearts_api::{
    Card, Cards, ChargeState, GameState, Rank, Seat, Suit, Suits, Trick, WonState,
};

const EMPTY: &'static [f32] = &[0.0, 0.0, 0.0, 0.0];

pub struct Encoder(Vec<f32>);

impl Encoder {
    pub const CARDS_LEN: usize = 208;
    pub const QUEEN_LEN: usize = 4;
    pub const JACK_LEN: usize = 4;
    pub const TEN_LEN: usize = 4;
    pub const HEARTS_LEN: usize = 4;
    pub const CHARGED_LEN: usize = 4;
    pub const LED_LEN: usize = 3;
    pub const TRICK_LEN: usize = 15;
    pub const PLAYS_LEN: usize = 52;

    pub fn new(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn into_inner(self) -> Vec<f32> {
        self.0
    }

    pub fn cards(mut self, seat: Seat, played: Cards, hands: [Cards; 4]) -> Self {
        let hands: [Cards; 4] = [
            hands[seat.idx()] - played,
            hands[seat.left().idx()] - played,
            hands[seat.across().idx()] - played,
            hands[seat.right().idx()] - played,
        ];
        for &suit in &Suit::VALUES {
            let chargeable = (suit.cards() & Cards::CHARGEABLE).max();
            let nine = suit.with_rank(Rank::Nine);
            let high = chargeable.above();
            let middle = nine.above() - high - chargeable;
            let low = nine.below();
            let mut cards = (high - played).into_iter();
            for _ in 0..high.len() {
                if let Some(card) = cards.next() {
                    for hand in &hands {
                        self.0.push(hand.contains(card) as i32 as f32);
                    }
                } else {
                    self.0.extend_from_slice(EMPTY);
                }
            }
            for hand in &hands {
                self.0.push(hand.contains(chargeable) as i32 as f32);
            }
            let mut cards = (middle - played).into_iter();
            for _ in 0..middle.len() {
                if let Some(card) = cards.next() {
                    for hand in &hands {
                        self.0.push(hand.contains(card) as i32 as f32);
                    }
                } else {
                    self.0.extend_from_slice(EMPTY);
                }
            }
            for hand in &hands {
                self.0.push(hand.contains(nine) as i32 as f32);
            }
            let mut cards = (low - played).into_iter();
            for _ in 0..low.len() {
                if let Some(card) = cards.next() {
                    for hand in &hands {
                        self.0.push(hand.contains(card) as i32 as f32);
                    }
                } else {
                    self.0.extend_from_slice(EMPTY);
                }
            }
        }
        self
    }

    pub fn queen(mut self, seat: Seat, won_state: WonState) -> Self {
        for &s in &[seat, seat.left(), seat.across(), seat.right()] {
            self.0.push(won_state.queen(s) as i32 as f32);
        }
        self
    }

    pub fn jack(mut self, seat: Seat, won_state: WonState) -> Self {
        for &s in &[seat, seat.left(), seat.across(), seat.right()] {
            self.0.push(won_state.jack(s) as i32 as f32);
        }
        self
    }

    pub fn ten(mut self, seat: Seat, won_state: WonState) -> Self {
        for &s in &[seat, seat.left(), seat.across(), seat.right()] {
            self.0.push(won_state.ten(s) as i32 as f32);
        }
        self
    }

    pub fn hearts(mut self, seat: Seat, won_state: WonState) -> Self {
        for &s in &[seat, seat.left(), seat.across(), seat.right()] {
            self.0.push(won_state.hearts(s) as f32 / 13.0);
        }
        self
    }

    pub fn charged(mut self, charges: ChargeState) -> Self {
        for card in Cards::CHARGEABLE {
            self.0.push(charges.is_charged(card) as i32 as f32);
        }
        self
    }

    pub fn led(mut self, led_suits: Suits) -> Self {
        self.0
            .push(led_suits.contains(Suit::Diamonds) as i32 as f32);
        self.0.push(led_suits.contains(Suit::Hearts) as i32 as f32);
        self.0.push(led_suits.contains(Suit::Spades) as i32 as f32);
        self
    }

    pub fn trick(mut self, seat: Seat, played: Cards, trick: Trick) -> Self {
        self.0.push(trick.len() as f32 / 7.0);
        let cards = trick.cards();
        let suit = trick.suit();
        for &s in &Suit::VALUES {
            self.0.push((s == suit) as i32 as f32);
        }

        let winning_card = (suit.cards() & cards).max();
        let chargeable = (suit.cards() & Cards::CHARGEABLE).max();
        let nine = suit.with_rank(Rank::Nine);
        let high = chargeable.above();
        let middle = nine.above() - high - chargeable;
        let low = nine.below();

        self.0.push(if high.is_empty() {
            0.0
        } else {
            (high - played).above(winning_card).len() as f32 / high.len() as f32
        });
        self.0.push((chargeable > winning_card) as i32 as f32);
        self.0.push(if middle.is_empty() {
            0.0
        } else {
            (middle - played).above(winning_card).len() as f32 / middle.len() as f32
        });
        self.0.push((nine > winning_card) as i32 as f32);
        self.0
            .push((low - played).above(winning_card).len() as f32 / low.len() as f32);

        self.0
            .push(cards.contains(winning_card.with_rank(Rank::Nine)) as i32 as f32);
        let winning_seat = trick.winning_seat(seat);
        for &s in &[seat, seat.left(), seat.across(), seat.right()] {
            self.0.push((s == winning_seat) as i32 as f32);
        }
        self
    }

    pub fn plays(mut self, hand: Cards, game_state: &GameState, plays: &[i16]) -> Self {
        let min = *plays.into_iter().min().unwrap();
        let max = *plays.into_iter().max().unwrap();
        if min == max {
            return self;
        }
        let avg = (min + max) as f32 / 2.0;
        let legal_plays = game_state.legal_plays(hand);
        let distinct_plays =
            legal_plays.distinct_plays(game_state.played, game_state.current_trick);
        let values: HashMap<Card, i16> = distinct_plays
            .into_iter()
            .zip(plays.into_iter().cloned())
            .collect();

        for &suit in &Suit::VALUES {
            let chargeable = (suit.cards() & Cards::CHARGEABLE).max();
            let nine = suit.with_rank(Rank::Nine);
            let high = chargeable.above();
            let middle = nine.above() - high - chargeable;
            let low = nine.below();
            let mut cards = (high & legal_plays).into_iter();
            for _ in 0..high.len() {
                if let Some(card) = cards.next() {
                    let value = values[&distinct(card, distinct_plays)];
                    self.0.push(value as f32 - avg);
                } else {
                    self.0.push(-1e8);
                }
            }
            if legal_plays.contains(chargeable) {
                let value = values[&chargeable];
                self.0.push(value as f32 - avg);
            } else {
                self.0.push(-1e8);
            }

            let mut cards = (middle & legal_plays).into_iter();
            for _ in 0..middle.len() {
                if let Some(card) = cards.next() {
                    let value = values[&distinct(card, distinct_plays)];
                    self.0.push(value as f32 - avg);
                } else {
                    self.0.push(-1e8);
                }
            }
            if legal_plays.contains(nine) {
                let value = values[&nine];
                self.0.push(value as f32 - avg);
            } else {
                self.0.push(-1e8);
            }

            let mut cards = (low & legal_plays).into_iter();
            for _ in 0..low.len() {
                if let Some(card) = cards.next() {
                    let value = values[&distinct(card, distinct_plays)];
                    self.0.push(value as f32 - avg);
                } else {
                    self.0.push(-1e8);
                }
            }
        }
        self
    }
}

fn distinct(card: Card, distinct: Cards) -> Card {
    if distinct.contains(card) {
        card
    } else {
        (card.above() & distinct).min()
    }
}
