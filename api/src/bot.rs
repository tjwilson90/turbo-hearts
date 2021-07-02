use crate::{Cards, GameEvent, GameState, Rank, Seat, Suit, VoidState};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotStrategy {
    Duck,
    GottaTry,
    Heuristic,
    Random,
    Simulate,
    NeuralNet,
}

#[derive(Clone, Debug)]
pub struct BotState {
    pub seat: Seat,
    pub pre_pass_hand: Cards,
    pub post_pass_hand: Cards,
    pub void: VoidState,
}

impl BotState {
    pub fn new(seat: Seat, hand: Cards) -> BotState {
        BotState::with_void(seat, hand, VoidState::new())
    }

    pub fn with_void(seat: Seat, hand: Cards, void: VoidState) -> BotState {
        BotState {
            seat,
            pre_pass_hand: hand,
            post_pass_hand: hand,
            void,
        }
    }

    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        self.void = self.void.on_event(state, event);
    }
}

pub fn should_claim(state: &GameState, void: VoidState, seat: Seat, hand: Cards) -> bool {
    if !state.current_trick.is_empty() {
        // checking claims in the middle of tricks is more expensive / not worth it
        false
    } else if state.claims.is_claiming(seat) {
        false
    } else if state.played.len() >= 48 {
        // don't bother claiming the last trick, more annoying than useful
        false
    } else if must_claim(hand - state.played, state.played) {
        true
    } else if !state.won.can_run(seat) {
        false
    } else {
        can_claim(state, void, seat, hand - state.played)
    }
}

fn must_claim(hand: Cards, played: Cards) -> bool {
    let remaining = Cards::ALL - hand - played;
    for suit in &[Cards::SPADES, Cards::HEARTS, Cards::DIAMONDS, Cards::CLUBS] {
        let hand = hand & *suit;
        let remaining = remaining & *suit;
        if !hand.is_empty() && !remaining.is_empty() && hand.min() < remaining.max() {
            return false;
        }
    }
    true
}

pub fn can_claim(state: &GameState, void: VoidState, seat: Seat, hand: Cards) -> bool {
    fn can_claim_rec(state: &GameState, void: VoidState, seat: Seat, hand: Cards) -> bool {
        if state.current_trick.is_empty() && state.next_actor == Some(seat) {
            return can_leader_claim(hand, state);
        }
        match state.next_actor {
            Some(actor) if seat == actor => {
                for card in state
                    .legal_plays(hand)
                    .distinct_plays(state.played, state.current_trick)
                    .shuffled()
                {
                    let mut state = state.clone();
                    state.apply(&GameEvent::Play { seat, card });
                    if state.current_trick.is_empty() && state.next_actor != Some(seat) {
                        continue;
                    }
                    if can_claim_rec(&state, void, seat, hand - card) {
                        return true;
                    }
                }
                false
            }
            Some(actor) => {
                for card in (Cards::ALL - state.played - hand)
                    .distinct_plays(state.played, state.current_trick)
                    .shuffled()
                    .filter(|c| !void.is_void(actor, c.suit()))
                {
                    let mut state = state.clone();
                    state.apply(&GameEvent::Play { seat: actor, card });
                    if state.current_trick.is_empty() && state.next_actor != Some(seat) {
                        return false;
                    }
                    if !can_claim_rec(&state, void, seat, hand) {
                        return false;
                    }
                }
                true
            }
            None => false,
        }
    }

    if (state.played - state.current_trick.cards()).contains_all(Cards::SCORING) {
        return true;
    }
    can_claim_rec(state, void, seat, hand)
}

fn can_leader_claim(hand: Cards, state: &GameState) -> bool {
    let heart_losers = losers(Suit::Hearts, hand, &state);
    let other_losers = losers(Suit::Spades, hand, &state)
        + losers(Suit::Diamonds, hand, &state)
        + losers(Suit::Clubs, hand, &state);
    if state.played.contains_any(Cards::HEARTS) {
        heart_losers + other_losers <= 0
    } else {
        other_losers <= 0 && heart_losers + other_losers <= 0
    }
}

fn losers(suit: Suit, hand: Cards, state: &GameState) -> i8 {
    let mut hand = (hand - state.played) & suit.cards();
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
        Card, ChargeState, ChargingRules, ClaimState, DoneState, GamePhase, GameState, Suits,
        Trick, WonState,
    };

    #[test]
    fn test_losers() {
        fn case(hand: &str, remaining: &str, can_lead_charged: bool) -> i8 {
            let hand = format!("{}S", hand).parse().unwrap();
            let remaining: Cards = format!("{}S", remaining).parse().unwrap();
            let mut state = GameState::new();
            state.played = Cards::SPADES - hand - remaining;
            if !can_lead_charged {
                state.charges = state.charges.charge(Seat::North, Cards::QUEEN_SPADES);
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
            &state,
            VoidState::new(),
            Seat::North,
            "AK9S AK9H AK9D AK9C".parse().unwrap(),
        ));
    }

    #[test]
    fn test_can_claim2() {
        let state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassLeft,
            done: DoneState::new(),
            charge_count: 2,
            charges: ChargeState::new()
                .charge(Seat::West, Card::TenClubs.into())
                .charge(Seat::North, Card::JackDiamonds.into()),
            next_actor: Some(Seat::West),
            played: "2K9C TS 584C TD  7D JC 68D  AKS".parse().unwrap(),
            claims: ClaimState::new(),
            won: WonState::new()
                .win(Seat::South, "2K9C TS 584C TD".parse().unwrap())
                .win(Seat::East, "7D JC 68D".parse().unwrap()),
            led_suits: Suits::NONE | Suit::Clubs | Suit::Diamonds | Suit::Spades,
            current_trick: Trick::new().push(Card::AceSpades).push(Card::KingSpades),
        };
        assert!(can_claim(
            &state,
            VoidState::new(),
            Seat::East,
            "AQJ98652S TH A8D 52C".parse().unwrap(),
        ));
    }

    #[test]
    fn test_can_claim3() {
        let state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassAcross,
            done: DoneState::new(),
            charge_count: 0,
            charges: ChargeState::new(),
            next_actor: Some(Seat::North),
            played: "2745C  K85TS  A6T4D  AT8JC  947S 9H Q36S 9C  AQH 3D 8H  KH 2S 9D 6H  TH JD 6C JH  53H"
                .parse()
                .unwrap(),
            claims: ClaimState::new(),
            won: WonState::new()
                .win(Seat::West, "2745C".parse().unwrap())
                .win(Seat::West, "K85TS".parse().unwrap())
                .win(Seat::West, "A6T4D".parse().unwrap())
                .win(Seat::West, "AT8JC".parse().unwrap())
                .win(Seat::West, "947S 9H Q36S 9C".parse().unwrap())
                .win(Seat::West, "AQH 3D 8H".parse().unwrap())
                .win(Seat::West, "KH 2S 9D 6H".parse().unwrap())
                .win(Seat::South, "TH JD 6C JH".parse().unwrap()),
            led_suits: Suits::NONE | Suit::Clubs | Suit::Diamonds | Suit::Hearts | Suit::Spades,
            current_trick: Trick::new().push(Card::FiveHearts).push(Card::ThreeHearts),
        };
        assert!(can_claim(
            &state,
            VoidState::new()
                .mark_void(Seat::South, Suit::Spades)
                .mark_void(Seat::East, Suit::Hearts)
                .mark_void(Seat::North, Suit::Hearts),
            Seat::South,
            "TS J9865H KQ4D KJ92C".parse().unwrap(),
        ));
    }
}
