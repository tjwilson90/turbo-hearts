use crate::TranspositionTable;
use turbo_hearts_api::{Cards, GameEvent, GameState, WonState};

pub struct BruteForce {
    hands: [Cards; 4],
    table: TranspositionTable<WonState>,
}

impl BruteForce {
    pub fn new(hands: [Cards; 4]) -> Self {
        Self {
            hands,
            table: TranspositionTable::new(hands),
        }
    }

    pub fn hands(&self) -> &[Cards; 4] {
        &self.hands
    }

    pub fn solve(&mut self, state: &mut GameState) -> WonState {
        if state.current_trick.is_empty() && state.played.contains_all(Cards::SCORING) {
            return state.won;
        }
        if state.played.len() >= 48 {
            while state.played != Cards::ALL {
                let seat = state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - state.played).max();
                state.apply(&GameEvent::Play { seat, card });
            }
            return state.won;
        }
        let seat = state.next_actor.unwrap();
        let plays = state
            .legal_plays(self.hands[seat.idx()])
            .distinct_plays(state.played, state.current_trick);
        let key = match plays.len() {
            1 => {
                state.apply(&GameEvent::Play {
                    seat,
                    card: plays.max(),
                });
                return self.solve(state);
            }
            2 => None,
            _ => match self.table.lookup(state) {
                Ok(won) => return won,
                Err(key) => Some(key),
            },
        };
        let mut best_won = WonState::new();
        let mut best_money = i16::MIN;
        for card in plays {
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
