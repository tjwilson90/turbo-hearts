use crate::{
    bot::{Algorithm, BotState},
    card::Card,
    cards::Cards,
    game::event::GameEvent,
    rank::Rank,
};
use rand::Rng;

macro_rules! check {
    ($hand:ident, $cards:expr, $len:literal) => {
        let cards = $cards.into();
        if $hand.contains_any(cards) && ($hand & cards.max().suit().cards()).len() <= $len {
            $hand -= ($hand & cards).max();
            continue;
        }
    };
}

pub struct Heuristic;

impl Heuristic {
    pub fn new() -> Self {
        Self
    }
}

impl Algorithm for Heuristic {
    fn pass(&mut self, state: &BotState) -> Cards {
        let mut hand = state.pre_pass_hand;
        if hand.contains_any(Cards::HEARTS) {
            if (hand & Cards::HEARTS).len() == 1 {
                hand -= Cards::HEARTS;
            } else {
                hand -= (hand & Cards::HEARTS).into_iter().nth(1).unwrap();
            }
        }
        while hand.len() > 10 {
            check!(hand, Card::QueenSpades, 2);
            check!(hand, Card::AceSpades, 2);
            check!(hand, Card::KingSpades, 2);
            check!(hand, Card::TenClubs, 2);
            check!(hand, Card::AceClubs, 2);
            check!(hand, Card::KingClubs, 2);
            check!(hand, Card::QueenClubs, 2);
            check!(hand, Card::JackClubs, 2);
            check!(hand, Cards::HEARTS, 1);
            check!(hand, Cards::CLUBS, 1);
            check!(hand, Cards::DIAMONDS, 1);
            check!(hand, Card::QueenSpades, 4);
            check!(hand, Card::AceSpades, 4);
            check!(hand, Card::KingSpades, 4);
            check!(hand, Card::TenClubs, 4);
            check!(hand, Card::AceClubs, 4);
            check!(hand, Card::KingClubs, 4);
            check!(hand, Card::QueenClubs, 4);
            check!(hand, Card::JackClubs, 4);
            check!(hand, Cards::HEARTS, 2);
            check!(hand, Cards::CLUBS, 2);
            check!(hand, Cards::DIAMONDS, 2);
            check!(hand, Cards::SPADES, 1);
            check!(hand, Cards::HEARTS, 13);
            check!(hand, Cards::CLUBS, 13);
            check!(hand, Cards::DIAMONDS, 13);
            check!(hand, Cards::SPADES, 13);
        }
        state.pre_pass_hand - hand
    }

    fn charge(&mut self, state: &BotState) -> Cards {
        let hand = state.post_pass_hand;
        let chargeable = hand - state.game.charged[state.seat.idx()];
        let mut charge = Cards::NONE;
        if chargeable.contains(Card::QueenSpades) {
            let spades = hand & Cards::SPADES;
            if spades.len() >= 6 || (spades.len() >= 5 && hand.contains(Card::NineSpades)) {
                if (hand - Cards::SPADES - Card::TwoClubs)
                    .into_iter()
                    .any(|c| c.rank() < Rank::Five)
                {
                    charge |= Card::QueenSpades;
                }
            }
        }
        if chargeable.contains(Card::TenClubs) {
            let clubs = hand & Cards::CLUBS;
            if clubs.len() >= 6 || (clubs.len() >= 5 && hand.contains(Card::NineClubs)) {
                if (hand - Cards::CLUBS)
                    .into_iter()
                    .any(|c| c.rank() < Rank::Five)
                {
                    charge |= Card::TenClubs;
                }
            }
        }
        if chargeable.contains(Card::AceHearts) {
            let hearts = hand & Cards::HEARTS;
            if hearts.below(Card::EightHearts).len() >= 3 {
                charge |= Card::AceHearts;
            }
        }
        if chargeable.contains(Card::JackDiamonds) {
            let diamonds = hand & Cards::DIAMONDS;
            let high_diamonds = diamonds.above(Card::JackDiamonds).len();
            let high_cards = (hand - Cards::DIAMONDS)
                .into_iter()
                .filter(|c| *c == Card::QueenSpades || c.rank() > Rank::Queen)
                .count();
            if high_diamonds == 3
                || (diamonds.len() >= 5 && high_diamonds == 2 && high_cards > 1)
                || (diamonds.len() >= 6 && high_diamonds == 1 && high_cards > 2)
            {
                charge |= Card::JackDiamonds;
            }
        }
        charge
    }

    fn play(&mut self, state: &BotState) -> Card {
        let cards = state.game.legal_plays(state.post_pass_hand);
        if cards.len() == 1 {
            return cards.max();
        }
        let remaining = Cards::ALL - state.post_pass_hand - state.game.played;
        if state.game.current_trick.is_empty() {
            let spades = cards & Cards::SPADES;
            if remaining.contains(Card::QueenSpades) && !spades.is_empty() {
                let low_spades = spades.below(Card::QueenSpades);
                let high_spades = spades.above(Card::QueenSpades);
                if high_spades.is_empty() || low_spades.len() >= 4 {
                    return random(low_spades);
                }
                if !high_spades.is_empty()
                    && state.game.charged_cards().contains(Card::QueenSpades)
                    && !state.game.led_suits.contains_any(Cards::SPADES)
                {
                    return random(high_spades);
                }
            }
            let clubs = cards & Cards::CLUBS;
            if remaining.contains(Card::TenClubs) && !clubs.is_empty() {
                let low_clubs = clubs.below(Card::TenClubs);
                let high_clubs = clubs.above(Card::TenClubs);
                if high_clubs.is_empty() || low_clubs.len() >= 4 {
                    return random(low_clubs);
                }
            }
        }
        random(cards)
    }

    fn on_event(&mut self, _: &BotState, _: &GameEvent) {}
}

fn random(cards: Cards) -> Card {
    let index = rand::thread_rng().gen_range(0, cards.len());
    cards.into_iter().nth(index).unwrap()
}
