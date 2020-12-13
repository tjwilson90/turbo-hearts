use turbo_hearts_api::{Card, Cards, ChargeState, Rank, Seat, Suit, Suits, Trick, WonState};

pub fn cards(seat: Seat, played: Cards, hands: [Cards; 4]) -> Vec<f32> {
    let mut vec = Vec::with_capacity(260);
    let all_cards = [
        hands[seat.idx()] - played,
        hands[seat.left().idx()] - played,
        hands[seat.across().idx()] - played,
        hands[seat.right().idx()] - played,
        played,
    ];
    for &cards in &all_cards {
        for card in Cards::ALL {
            vec.push(cards.contains(card) as i32 as f32);
        }
    }
    vec
}

pub fn queen(seat: Seat, won_state: WonState) -> Vec<f32> {
    let mut vec = Vec::with_capacity(4);
    for &s in &[seat, seat.left(), seat.across(), seat.right()] {
        vec.push(won_state.queen(s) as i32 as f32);
    }
    vec
}

pub fn jack(seat: Seat, won_state: WonState) -> Vec<f32> {
    let mut vec = Vec::with_capacity(4);
    for &s in &[seat, seat.left(), seat.across(), seat.right()] {
        vec.push(won_state.jack(s) as i32 as f32);
    }
    vec
}

pub fn ten(seat: Seat, won_state: WonState) -> Vec<f32> {
    let mut vec = Vec::with_capacity(4);
    for &s in &[seat, seat.left(), seat.across(), seat.right()] {
        vec.push(won_state.ten(s) as i32 as f32);
    }
    vec
}

pub fn hearts(seat: Seat, won_state: WonState) -> Vec<f32> {
    let mut vec = Vec::with_capacity(4);
    for &s in &[seat, seat.left(), seat.across(), seat.right()] {
        vec.push(won_state.hearts(s) as f32 / 13.0);
    }
    vec
}

pub fn charged(charges: ChargeState) -> Vec<f32> {
    let mut vec = Vec::with_capacity(4);
    for card in Cards::CHARGEABLE {
        vec.push(charges.is_charged(card) as i32 as f32);
    }
    vec
}

pub fn led(led_suits: Suits) -> Vec<f32> {
    let mut vec = Vec::with_capacity(3);
    vec.push(led_suits.contains(Suit::Diamonds) as i32 as f32);
    vec.push(led_suits.contains(Suit::Hearts) as i32 as f32);
    vec.push(led_suits.contains(Suit::Spades) as i32 as f32);
    vec
}

pub fn trick(seat: Seat, trick: Trick) -> Vec<f32> {
    let mut vec = Vec::with_capacity(62);
    vec.push(trick.len() as f32 / 7.0);
    let cards = trick.cards();
    vec.push(cards.contains(Card::QueenSpades) as i32 as f32);
    vec.push(cards.contains(Card::JackDiamonds) as i32 as f32);
    vec.push(cards.contains(Card::TenClubs) as i32 as f32);
    vec.push((cards & Cards::HEARTS).len() as f32 / 7.0);
    let winning_card = (cards & trick.suit().cards()).max();
    for card in Cards::ALL {
        vec.push((winning_card == card) as i32 as f32);
    }
    vec.push(cards.contains(winning_card.with_rank(Rank::Nine)) as i32 as f32);
    let winning_seat = trick.winning_seat(seat);
    for &s in &[seat, seat.left(), seat.across(), seat.right()] {
        vec.push((s == winning_seat) as i32 as f32);
    }
    vec
}
