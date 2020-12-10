use crate::TranspositionTable;
use turbo_hearts_api::{Card, Cards, GameEvent, GameState, WonState};

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

    pub fn solve(&mut self, state: &mut GameState) -> (Card, WonState) {
        if state.played.len() >= 48 {
            while state.played != Cards::ALL {
                let seat = state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - state.played).max();
                state.apply(&GameEvent::Play { seat, card });
            }
            return (Card::TwoClubs, state.won);
        }
        let seat = state.next_actor.unwrap();
        let key = if state.current_trick.is_empty() {
            match self
                .table
                .lookup(seat, state.led_suits, state.won, state.played)
            {
                (_, Some(results)) => return results,
                (key, _) => Some(key),
            }
        } else {
            None
        };
        let plays = state.legal_plays(self.hands[seat.idx()]);
        let mut best_play = Card::TwoClubs;
        let mut best_won = WonState::new();
        let mut best_money = -1000;
        for card in plays.distinct_plays(state.played, state.current_trick) {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let (_, opt_won) = self.solve(&mut state);
            let opt_money = opt_won.scores(state.charges.all_charges()).money(seat);
            if opt_money > best_money {
                best_play = card;
                best_won = opt_won;
                best_money = opt_money;
            }
        }
        if let Some(key) = key {
            self.table.cache(key, best_play, best_won);
        }
        (best_play, best_won)
    }
}
