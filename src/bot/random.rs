use crate::{
    bot::{Algorithm, BotState},
    cards::{Card, Cards},
    types::Seat,
};
use rand::{seq::SliceRandom, Rng};

pub struct Random {
    charged: bool,
}

impl Random {
    pub const NAME: &'static str = "random";

    pub fn new() -> Self {
        Self { charged: false }
    }
}

impl Algorithm for Random {
    fn pass(&mut self, state: &BotState) -> Cards {
        let mut hand = state.pre_pass_hand.into_iter().collect::<Vec<_>>();
        hand.partial_shuffle(&mut rand::thread_rng(), 3);
        hand.into_iter().take(3).collect()
    }

    fn charge(&mut self, state: &BotState) -> Cards {
        if self.charged {
            return Cards::NONE;
        }
        let cards =
            (state.post_pass_hand & Cards::CHARGEABLE) - state.game.charged[state.seat.idx()];
        cards
            .into_iter()
            .filter(|_| rand::thread_rng().gen())
            .collect()
    }

    fn play(&mut self, state: &BotState) -> Card {
        let cards = state.game.legal_plays(state.post_pass_hand);
        let index = rand::thread_rng().gen_range(0, cards.len());
        cards.into_iter().nth(index).unwrap()
    }

    fn on_deal(&mut self, _: &BotState) {
        self.charged = false;
    }

    fn on_charge(&mut self, state: &BotState, seat: Seat, _: Cards) {
        self.charged |= seat == state.seat;
    }

    fn on_blind_charge(&mut self, state: &BotState, seat: Seat, _: usize) {
        self.charged |= seat == state.seat;
    }
}
