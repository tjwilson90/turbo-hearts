use crate::RandomBot;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState};

pub struct DuckBot;

impl DuckBot {
    pub fn new() -> Self {
        Self
    }
}

impl DuckBot {
    pub async fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        let mut pass = Cards::NONE;
        let mut hand = bot_state.pre_pass_hand;
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

    pub async fn charge(&mut self, _: &BotState, _: &GameState) -> Cards {
        Cards::NONE
    }

    pub async fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if game_state.current_trick.is_empty() {
            return RandomBot::new().play(bot_state, game_state).await;
        }
        let suit = game_state.current_trick.suit();
        if !suit.cards().contains_any(cards) {
            return cards
                .into_iter()
                .max_by_key(|card| score(*card, bot_state.post_pass_hand - game_state.played))
                .unwrap();
        }
        let winner = (game_state.current_trick.cards() & suit.cards()).max();
        let duck = cards.into_iter().filter(|card| *card < winner).max();
        match duck {
            Some(card) => card,
            _ => cards.into_iter().max().unwrap(),
        }
    }

    pub fn on_event(&mut self, _: &BotState, _: &GameState, _: &GameEvent) {}
}

fn score(card: Card, hand: Cards) -> usize {
    14 * card.rank() as usize + 13 - (card.suit().cards() & hand).len()
}
