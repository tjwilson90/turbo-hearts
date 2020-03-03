use crate::{
    bot::{duck::Duck, random::Random, Algorithm, BotState},
    card::Card,
    cards::Cards,
    game::event::GameEvent,
};

pub struct GottaTry;

impl GottaTry {
    pub fn new() -> Self {
        Self
    }
}

impl Algorithm for GottaTry {
    fn pass(&mut self, state: &BotState) -> Cards {
        let mut pass = Cards::NONE;
        let mut hand = state.pre_pass_hand;
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

    fn charge(&mut self, state: &BotState) -> Cards {
        (state.post_pass_hand & Cards::CHARGEABLE) - state.game.charged_cards()
    }

    fn play(&mut self, state: &BotState) -> Card {
        let cards = state.game.legal_plays(state.post_pass_hand);
        let our_won = state.game.won[state.seat.idx()];
        let all_won = state.game.played;
        if Cards::POINTS.contains_any(all_won - our_won) {
            return Duck::new().play(state);
        }
        let trick = &state.game.current_trick;
        if trick.is_empty() {
            return Random::new().play(state);
        }
        let trick_cards = trick.iter().cloned().collect();
        if Cards::POINTS.contains_any(trick_cards) {
            if !trick[0].suit().cards().contains_any(cards) {
                Duck::new().play(state)
            } else {
                cards
                    .into_iter()
                    .max_by_key(|card| score(*card, state.post_pass_hand - state.game.played))
                    .unwrap()
            }
        } else {
            cards
                .into_iter()
                .min_by_key(|card| score(*card, state.post_pass_hand - state.game.played))
                .unwrap()
        }
    }

    fn on_event(&mut self, _: &BotState, _: &GameEvent) {}
}

fn score(card: Card, hand: Cards) -> usize {
    14 * card.rank() as usize + (card.suit().cards() & hand).len()
}
