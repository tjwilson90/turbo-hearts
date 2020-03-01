use crate::{
    bot::{random::Random, Algorithm, BotState},
    cards::{Card, Cards},
    game::GameEvent,
};

pub struct Duck;

impl Duck {
    pub const NAME: &'static str = "duck";

    pub fn new() -> Self {
        Self
    }
}

impl Algorithm for Duck {
    fn pass(&mut self, state: &BotState) -> Cards {
        let mut pass = Cards::NONE;
        let mut hand = state.pre_pass_hand;
        for _ in 0..3 {
            let worst_card = hand
                .into_iter()
                .max_by_key(|card| score(*card, hand))
                .unwrap();
            pass |= worst_card;
            hand -= worst_card;
        }
        pass
    }

    fn charge(&mut self, _: &BotState) -> Cards {
        Cards::NONE
    }

    fn play(&mut self, state: &BotState) -> Card {
        let cards = state.game.legal_plays(state.post_pass_hand);
        if state.game.current_trick.is_empty() {
            return Random::new().play(state);
        }
        let suit = state.game.current_trick[0].suit();
        if !suit.cards().contains_any(cards) {
            return cards
                .into_iter()
                .max_by_key(|card| score(*card, state.post_pass_hand - state.game.played))
                .unwrap();
        }
        let winner = state
            .game
            .current_trick
            .iter()
            .filter(|card| card.suit() == suit)
            .max()
            .unwrap();
        let duck = cards.into_iter().filter(|card| card < winner).max();
        match duck {
            Some(card) => card,
            _ => cards.into_iter().max().unwrap(),
        }
    }

    fn on_event(&mut self, _: &BotState, _: &GameEvent) {}
}

fn score(card: Card, hand: Cards) -> usize {
    14 * card.rank() as usize + 13 - (card.suit().cards() & hand).len()
}
