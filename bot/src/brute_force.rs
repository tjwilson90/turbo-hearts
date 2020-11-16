use crate::TranspositionTable;
use turbo_hearts_api::{Card, Cards, GameEvent, GameState, Seat, WonState};

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

    pub fn solve(&mut self, state: GameState) -> (Card, [i16; 4]) {
        self.solve_rec(state, Seat::North)
    }

    fn solve_rec(&mut self, state: GameState, prev_seat: Seat) -> (Card, [i16; 4]) {
        if state.played.len() >= 48 {
            let mut state = state;
            while state.played != Cards::ALL {
                let seat = state.next_actor.unwrap();
                let card = (self.hands[seat.idx()] - state.played).max();
                state.apply(&GameEvent::Play { seat, card });
            }
            return (
                Card::TwoClubs,
                money(state.won, state.charges.all_charges()),
            );
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
        let played = if state.current_trick.is_empty() {
            state.played
        } else {
            state.played - (state.current_trick.cards() & state.current_trick.suit().cards()).max()
        };
        let plays = plays.distinct_plays(played);
        let mut best_play = Card::TwoClubs;
        let mut best_scores = [-2000; 4];
        for card in plays.shuffled() {
            let mut state = state.clone();
            state.apply(&GameEvent::Play { seat, card });
            let prev = if seat == state.next_actor.unwrap() {
                prev_seat
            } else {
                seat
            };
            let (_, opt_scores) = self.solve_rec(state, prev);
            if opt_scores[seat.idx()] > best_scores[seat.idx()] {
                best_scores = opt_scores;
                best_play = card;
            }
        }
        if let Some(key) = key {
            self.table.cache(key, best_play, best_scores);
        }
        (best_play, best_scores)
    }
}

fn money(won: WonState, charges: Cards) -> [i16; 4] {
    let north = won.score(Seat::North, charges);
    let east = won.score(Seat::East, charges);
    let south = won.score(Seat::South, charges);
    let west = won.score(Seat::West, charges);
    let total = north + east + south + west;
    [
        total - 4 * north,
        total - 4 * east,
        total - 4 * south,
        total - 4 * west,
    ]
}
