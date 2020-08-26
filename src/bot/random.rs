use crate::{
    bot::{Algorithm, BotState},
    card::Card,
    cards::Cards,
    game::{event::GameEvent, state::GameState},
};
use rand::{seq::SliceRandom, Rng};

pub struct Random {
    charged: bool,
}

impl Random {
    pub fn new() -> Self {
        Self { charged: false }
    }
}

impl Algorithm for Random {
    fn pass(&mut self, bot_state: &BotState, _: &GameState) -> Cards {
        let mut hand = bot_state.pre_pass_hand.into_iter().collect::<Vec<_>>();
        hand.partial_shuffle(&mut rand::thread_rng(), 3);
        hand.into_iter().take(3).collect()
    }

    fn charge(&mut self, bot_state: &BotState, game_state: &GameState) -> Cards {
        if self.charged {
            return Cards::NONE;
        }
        let cards = (bot_state.post_pass_hand & Cards::CHARGEABLE)
            - game_state.charges.charges(bot_state.seat);
        cards
            .into_iter()
            .filter(|_| rand::thread_rng().gen())
            .collect()
    }

    fn play(&mut self, bot_state: &BotState, game_state: &GameState) -> Card {
        let cards = game_state.legal_plays(bot_state.post_pass_hand);
        let index = rand::thread_rng().gen_range(0, cards.len());
        cards.into_iter().nth(index).unwrap()
    }

    fn on_event(&mut self, state: &BotState, _: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Deal { .. } => {
                self.charged = false;
            }
            GameEvent::Charge { seat, .. } => {
                self.charged |= state.seat == *seat;
            }
            GameEvent::BlindCharge { seat, .. } => {
                self.charged |= state.seat == *seat;
            }
            _ => {}
        }
    }
}
