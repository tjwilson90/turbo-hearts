use crate::{Algorithm, DuckBot};
use rand::Rng;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GamePhase, GameState, Rank, Suit};

pub struct GottaTryBot;

fn can_drain_without_nine(suit: Suit, mut ours: Cards, mut theirs: Cards) -> bool {
    ours &= suit.cards();
    theirs &= suit.cards();
    if ours.is_empty() {
        return false;
    }
    while !ours.is_empty() {
        if theirs.is_empty() {
            return true;
        }
        if theirs.max() > ours.max() {
            return false;
        }
        ours -= ours.max();
        theirs -= theirs.min();
    }
    true
}

fn can_drain_with_nine(suit: Suit, mut ours: Cards, mut theirs: Cards) -> bool {
    ours &= suit.cards();
    theirs &= suit.cards();
    if !ours.contains(suit.with_rank(Rank::Nine)) {
        return false;
    }
    if theirs.is_empty() {
        return true;
    }
    if ours.len() <= 2 {
        return ours.max() > theirs.max();
    }
    can_drain_with_nine(
        suit,
        ours - suit.with_rank(Rank::Nine),
        theirs - theirs.min(),
    )
}

fn lead(ours: Cards, theirs: Cards) -> Cards {
    for suit in [Suit::Diamonds, Suit::Spades] {
        let us = ours & suit.cards();
        if !us.is_empty() && (theirs & suit.cards()).len() >= 7 {
            return us.min().into();
        }
    }
    for &suit in &Suit::VALUES {
        if can_drain_without_nine(suit, ours - suit.with_rank(Rank::Nine), theirs) {
            let nine = suit.with_rank(Rank::Nine);
            let us = (ours & suit.cards()) - nine;
            return us.max().into();
        }
    }
    for &suit in &Suit::VALUES {
        if can_drain_with_nine(suit, ours, theirs) {
            let nine = suit.with_rank(Rank::Nine);
            let us = (ours & suit.cards()) - nine;
            let them = theirs & suit.cards();
            if us.len() > 1 && (them.is_empty() || (us - us.max()).max() > them.max()) {
                return us.max().into();
            }
            return nine.into();
        }
    }
    for &suit in &Suit::VALUES {
        let nine = suit.with_rank(Rank::Nine);
        let us = ours & suit.cards();
        let them = theirs & suit.cards();
        if us == Cards::from(nine) && (them.is_empty() || nine > them.max()) {
            return nine.into();
        }
    }
    for &suit in &Suit::VALUES {
        let us = ours & suit.cards();
        let them = theirs & suit.cards();
        if !us.is_empty() && (them.is_empty() || us.max() > them.max()) {
            return us.max().into();
        }
    }
    ours
}

fn follow(ours: Cards, theirs: Cards, bot_state: &BotState, game_state: &GameState) -> Cards {
    let trick: Cards = game_state.current_trick.cards();
    let suit = game_state.current_trick.suit();
    let winning = game_state.current_trick.winning_seat(bot_state.seat) == bot_state.seat;
    let winning_card = (trick & suit.cards()).max();

    if !winning && trick.contains_any(Cards::POINTS) && ours.max() < winning_card {
        return ours.max().into();
    }

    if trick.len() == 4 && can_drain_without_nine(suit, ours, theirs) {
        return ours.max().into();
    }

    if trick.len() == 3 || trick.len() == 7 || (theirs & suit.cards()).len() >= 5 {
        if winning || !trick.contains_any(Cards::POINTS) {
            return ours.min().into();
        }
    }
    let winners = ours.above(winning_card);
    if !winners.is_empty() {
        return winners.min().into();
    }
    ours.min().into()
}

fn slough(ours: Cards, theirs: Cards, bot_state: &BotState, game_state: &GameState) -> Cards {
    let trick: Cards = game_state.current_trick.cards();
    let suit = game_state.current_trick.suit();
    let winning = game_state.current_trick.winning_seat(bot_state.seat) == bot_state.seat;
    let winning_card = (trick & suit.cards()).max();

    if !winning && trick.contains_any(Cards::POINTS) {
        return DuckBot.play(bot_state, game_state).into();
    }

    if Cards::POINTS.contains_all(ours) {
        return if winning {
            ours.min().into()
        } else {
            ours.max().into()
        };
    }

    let them = theirs & suit.cards();
    let options = if winning
        && (trick.len() == 3 || trick.len() == 7 || them.is_empty() || winning_card > them.max())
    {
        ours
    } else {
        ours - Cards::POINTS
    };
    worst(options, theirs).into()
}

impl Algorithm for GottaTryBot {
    fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        let mut pass = Cards::NONE;
        let mut hand = bot_state.pre_pass_hand;
        for _ in 0..3 {
            let worst_card = worst(hand, Cards::ALL - hand - pass);
            pass |= worst_card;
            hand -= worst_card;
        }
        pass
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        if game_state.phase == GamePhase::ChargeKeeper1
            && game_state.charges.all_charges().is_empty()
        {
            Cards::NONE
        } else {
            (bot_state.post_pass_hand & Cards::CHARGEABLE) - game_state.charges.all_charges()
        }
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        if !game_state.won.can_run(bot_state.seat) {
            return DuckBot.play(bot_state, game_state);
        }
        let ours = game_state.legal_plays(bot_state.post_pass_hand);
        if ours.len() == 1 {
            return ours.max();
        }
        let theirs = Cards::ALL - bot_state.post_pass_hand - game_state.played;
        let good_plays = if game_state.current_trick.is_empty() {
            lead(ours, theirs)
        } else if game_state.current_trick.suit().cards().contains_any(ours) {
            follow(ours, theirs, bot_state, game_state)
        } else {
            slough(ours, theirs, bot_state, game_state)
        };
        let index = rand::thread_rng().gen_range(0..good_plays.len());
        good_plays.into_iter().nth(index).unwrap()
    }

    fn on_event(&mut self, _: &BotState, _: &GameState, _: &GameEvent) {}
}

fn worst(ours: Cards, theirs: Cards) -> Card {
    ours.into_iter()
        .max_by_key(|&c| {
            let us = ours & c.suit().cards();
            let them = theirs & c.suit().cards();
            if them.is_empty() {
                return -13;
            }
            if c.rank() == Rank::Nine && us.max() > them.max() {
                return -13;
            }
            them.above(c).len() as isize - ours.above(them.max()).len() as isize
        })
        .unwrap()
        .into()
}

#[cfg(test)]
mod test {
    use super::*;
    use turbo_hearts_api::{
        ChargeState, ChargingRules, ClaimState, DoneState, GamePhase, Seat, Suits, Trick,
        VoidState, WonState,
    };

    #[test]
    fn test_play() {
        let bot_state = BotState {
            seat: Seat::South,
            pre_pass_hand: "A96S A6H KQ3D AKQT5C".parse().unwrap(),
            post_pass_hand: "A9S AJH AKQD AKQJT5C".parse().unwrap(),
            void: VoidState::new(),
        };
        let game_state = GameState {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassRight,
            done: DoneState::new(),
            charge_count: 0,
            charges: ChargeState::new()
                .charge(Seat::South, "AH TC".parse().unwrap())
                .charge(Seat::East, "QS".parse().unwrap()),
            next_actor: Some(Seat::South),
            played: "2975648JC QTJ3D K856D 9875AKT6S A4D TH 7D".parse().unwrap(),
            claims: ClaimState::new(),
            won: WonState::new().win(
                Seat::South,
                "2975648JC QTJ3D K856D 9875AKT6S A4D TH 7D".parse().unwrap(),
            ),
            led_suits: Suits::NONE | Suit::Clubs | Suit::Diamonds | Suit::Spades,
            current_trick: Trick::new(),
        };
        assert_eq!(Card::AceClubs, GottaTryBot.play(&bot_state, &game_state));
    }
}
