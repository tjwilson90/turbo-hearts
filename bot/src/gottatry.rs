use crate::{DuckBot, RandomBot};
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState};

pub struct GottaTryBot;

impl GottaTryBot {
    pub fn new() -> Self {
        Self
    }
}

impl GottaTryBot {
    pub async fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        let mut pass = Cards::NONE;
        let mut hand = bot_state.pre_pass_hand;
        for _ in 0..3 {
            let worst_card = hand
                .into_iter()
                .min_by_key(|card| score(*card, hand))
                .unwrap();
            pass |= worst_card;
            hand -= worst_card;
        }
        pass
    }

    pub async fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        (bot_state.post_pass_hand & Cards::CHARGEABLE) - game_state.charges.all_charges()
    }

    pub async fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if !game_state.won.can_run(bot_state.seat) {
            return DuckBot::new().play(bot_state, game_state).await;
        }
        if game_state.current_trick.is_empty() {
            return RandomBot::new().play(bot_state, game_state).await;
        }
        let trick_cards = game_state.current_trick.cards();
        if Cards::POINTS.contains_any(trick_cards) {
            if !game_state.current_trick.suit().cards().contains_any(cards) {
                DuckBot::new().play(bot_state, game_state).await
            } else {
                cards
                    .into_iter()
                    .max_by_key(|card| score(*card, bot_state.post_pass_hand - game_state.played))
                    .unwrap()
            }
        } else {
            cards
                .into_iter()
                .min_by_key(|card| score(*card, bot_state.post_pass_hand - game_state.played))
                .unwrap()
        }
    }

    pub fn on_event(&mut self, _: &BotState, _: &GameState, _: &GameEvent) {}
}

fn score(card: Card, hand: Cards) -> usize {
    14 * card.rank() as usize + (card.suit().cards() & hand).len()
}
