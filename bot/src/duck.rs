use crate::Algorithm;
use rand::RngExt;
use turbo_hearts_api::{BotState, Card, Cards, GameEvent, GameState};

pub struct DuckBot;

impl DuckBot {
    pub(crate) fn choices(&self, bot: &BotState, state: &GameState) -> Cards {
        let cards = state.legal_plays(bot.post_pass_hand);
        if state.current_trick.is_empty() {
            return cards;
        }
        let suit = state.current_trick.suit();
        let card = if !suit.cards().contains_any(cards) {
            cards
                .into_iter()
                .max_by_key(|card| score(*card, bot.post_pass_hand - state.played))
                .unwrap()
        } else {
            let winner = (state.current_trick.cards() & suit.cards()).max();
            cards
                .into_iter()
                .filter(|card| *card < winner)
                .max()
                .unwrap_or_else(|| cards.max())
        };
        card.into()
    }

    pub fn play_with_rng<R: RngExt + ?Sized>(
        &self,
        bot_state: &BotState,
        game_state: &GameState,
        rng: &mut R,
    ) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        if game_state.current_trick.is_empty() {
            return cards
                .into_iter()
                .nth(rng.random_range(0..cards.len()))
                .unwrap();
        }
        self.choices(bot_state, game_state).max()
    }
}

impl Algorithm for DuckBot {
    fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
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

    fn charge(&mut self, _: &BotState, _: &GameState) -> Cards {
        Cards::NONE
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        self.play_with_rng(bot_state, game_state, &mut rand::rng())
    }

    fn on_event(&mut self, _: &BotState, _: &GameState, _: &GameEvent) {}
}

fn score(card: Card, hand: Cards) -> usize {
    14 * card.rank() as usize + 13 - (card.suit().cards() & hand).len()
}
