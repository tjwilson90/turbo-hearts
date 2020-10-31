use crate::{Cards, GameEvent, GameState, Rank, Seat, Suit};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum BotStrategy {
    Duck,
    GottaTry,
    Heuristic,
    Random,
    Simulate,
}

pub struct BotState {
    pub seat: Seat,
    pub pre_pass_hand: Cards,
    pub post_pass_hand: Cards,
}

pub fn can_claim(seat: Seat, hand: Cards, state: &GameState) -> bool {
    if state.current_trick.is_empty() && state.next_actor == Some(seat) {
        return can_leader_claim(hand, state);
    }
    match state.next_actor {
        Some(actor) if seat == actor => {
            for card in state.legal_plays(hand).distinct_plays(state.played) {
                let mut state = state.clone();
                state.apply(&GameEvent::Play { seat, card });
                if state.current_trick.is_empty() && state.next_actor != Some(seat) {
                    return false;
                }
                if can_claim(seat, hand - card, &state) {
                    return true;
                }
            }
            false
        }
        Some(actor) => {
            for card in (Cards::ALL - state.played - hand).distinct_plays(state.played) {
                let mut state = state.clone();
                state.apply(&GameEvent::Play { seat: actor, card });
                if state.current_trick.is_empty() && state.next_actor != Some(seat) {
                    return false;
                }
                if !can_claim(seat, hand, &state) {
                    return false;
                }
            }
            true
        }
        None => false,
    }
}

fn can_leader_claim(hand: Cards, state: &GameState) -> bool {
    let heart_losers = losers(Suit::Hearts, hand, &state);
    let other_losers = losers(Suit::Spades, hand, &state)
        + losers(Suit::Diamonds, hand, &state)
        + losers(Suit::Clubs, hand, &state);
    return if state.played.contains_any(Cards::HEARTS) {
        heart_losers + other_losers <= 0
    } else {
        other_losers <= 0 && heart_losers + other_losers <= 0
    };
}

fn losers(suit: Suit, hand: Cards, state: &GameState) -> i8 {
    let mut hand = hand & suit.cards();
    let mut remaining = suit.cards() - hand - state.played;
    let nine = suit.with_rank(Rank::Nine);
    let mut legal_plays = if hand.len() == 1 || state.led_suits.contains(suit) {
        hand
    } else {
        hand - state.charges.all_charges()
    };
    let mut had_winner = false;

    loop {
        if hand.is_empty() {
            return 0;
        }
        if remaining.is_empty() {
            return if hand.contains(nine) { -1 } else { 0 };
        }
        let top = if remaining == nine.into() {
            nine
        } else {
            (remaining - nine).max()
        };
        if top > legal_plays.max() {
            if had_winner && hand.contains(nine) {
                hand -= nine;
                remaining -= remaining.min();
                continue;
            }
            if top == nine && hand.len() == 2 && hand.max() > top {
                return 0;
            }
            return if top > hand.max() {
                hand.len()
            } else {
                legal_plays.len()
            } as i8;
        }
        let winners: Cards = legal_plays.into_iter().filter(|c| *c > top).collect();
        if winners == nine.into() {
            hand -= nine;
            remaining -= remaining.min();
            if hand.is_empty() {
                return -1;
            }
            hand -= hand.min();
            if !remaining.is_empty() {
                remaining -= remaining.min();
            }
        } else {
            hand -= (winners - nine).min();
            remaining -= remaining.min();
        }
        legal_plays = hand;
        had_winner = true;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ChargeState, ChargingRules, ClaimState, DoneState, GamePhase, GameState, Suits, Trick,
        WonState,
    };

    #[test]
    fn test_losers() {
        fn case(hand: &str, remaining: &str, can_lead_charged: bool) -> i8 {
            let hand = format!("{}S", hand).parse().unwrap();
            let remaining: Cards = format!("{}S", remaining).parse().unwrap();
            let mut state = GameState::new();
            state.played = Cards::SPADES - hand - remaining;
            if !can_lead_charged {
                state.charges.charge(Seat::North, Cards::QUEEN_SPADES);
            }
            losers(Suit::Spades, hand, &state)
        }
        assert_eq!(0, case("AKQ", "23", true));
        assert_eq!(0, case("AK", "234", true));
        assert_eq!(-1, case("AK9", "23", true));
        assert_eq!(-1, case("A9", "23", true));
        assert_eq!(-1, case("A9", "234", true));
        assert_eq!(0, case("A92", "34", true));
        assert_eq!(-1, case("A92", "3", true));
        assert_eq!(0, case("A", "9", true));
        assert_eq!(-1, case("A9", "J", true));
        assert_eq!(0, case("A9", "QJ", true));
        assert_eq!(0, case("A9", "KQJ", true));
        assert_eq!(1, case("A93", "KQJ", true));
        assert_eq!(2, case("A83", "KQJ", true));
        assert_eq!(0, case("A93", "QJ", true));
        assert_eq!(0, case("J8", "954", true));
        assert_eq!(1, case("J8", "T954", true));
        assert_eq!(3, case("K93", "AQJ", true));
        assert_eq!(2, case("K3", "AQJ", true));
        assert_eq!(1, case("8", "9", true));

        assert_eq!(1, case("Q8", "T", false));
        assert_eq!(0, case("Q8", "9", false));
        assert_eq!(0, case("Q8", "4", false));
        assert_eq!(0, case("Q9", "4", false));
        assert_eq!(0, case("Q93", "4", false));
        assert_eq!(-1, case("Q95", "4", false));
        assert_eq!(0, case("Q", "T", false));
        assert_eq!(1, case("Q", "K", false));
    }

    #[test]
    fn test_can_claim() {
        let state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassLeft,
            done: DoneState::new(),
            charge_count: 0,
            charges: ChargeState::new(),
            next_actor: Some(Seat::East),
            played: Cards::NONE,
            claims: ClaimState::new(),
            won: WonState::new(),
            led_suits: Suits::NONE,
            current_trick: Trick::new(),
        };
        assert!(can_claim(
            Seat::North,
            "AK9S AK9H AK9D AK9C".parse().unwrap(),
            &state
        ));
    }
}
