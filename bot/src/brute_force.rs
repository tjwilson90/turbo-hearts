use crate::TranspositionTable;
use turbo_hearts_api::{Cards, GameEvent, GameState, WonState};

pub struct BruteForce {
    hands: [Cards; 4],
    table: TranspositionTable,
}

impl BruteForce {
    pub fn new(hands: [Cards; 4]) -> Self {
        Self {
            hands,
            table: TranspositionTable::new(hands),
        }
    }

    pub fn solve(&mut self, state: &mut GameState) -> WonState {
        if state.played.len() >= 48 {
            while state.played != Cards::ALL {
                let seat = state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - state.played).max();
                state.apply(&GameEvent::Play { seat, card });
            }
            return state.won;
        }
        let seat = state.next_actor.unwrap();
        let key = if state.current_trick.is_empty() {
            match self
                .table
                .lookup(seat, state.led_suits, state.won, state.played)
            {
                Ok(won) => return won,
                Err(key) => Some(key),
            }
        } else {
            None
        };
        let mut best_won = WonState::new();
        let mut best_money = -1000;
        for card in state
            .legal_plays(self.hands[seat.idx()])
            .distinct_plays(state.played, state.current_trick)
        {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let won = self.solve(&mut state);
            let money = won.scores(state.charges).money(seat);
            if money > best_money {
                best_won = won;
                best_money = money;
            }
        }
        if let Some(key) = key {
            self.table.cache(key, best_won);
        }
        best_won
    }
}
