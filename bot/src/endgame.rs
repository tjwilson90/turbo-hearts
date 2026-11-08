//! Reuse exact max-n solutions across equivalent remaining deals.
use crate::neural_network::solve_endgame;
use std::collections::HashMap;
use turbo_hearts_api::{Cards, GameEvent, GameState, Rank, Seat, Suit, WonState};

#[derive(Hash, PartialEq, Eq)]
struct Key {
    remaining: [u64; 4],
    trick: u64,
    won: WonState,
    next: Seat,
    led: u8,
}

/// One solver per real decision: charges are constant throughout its lifetime.
/// Hands can vary across sampled deals; remaining ownership is part of the key.
#[derive(Default)]
pub(crate) struct EndgameSolver {
    cache: HashMap<Key, WonState>,
    canonical: bool,
}

impl EndgameSolver {
    pub(crate) fn new(canonical: bool) -> Self {
        Self {
            canonical,
            ..Self::default()
        }
    }
    pub(crate) fn solve(&mut self, state: &GameState, hands: &[Cards; 4]) -> WonState {
        // Tiny subtrees are cheaper to traverse than hash. This also bounds
        // cache growth without changing the answer or max-n tie breaking.
        if state.played.len() >= 44 {
            return solve_endgame(state, hands);
        }
        if state.current_trick.is_empty() && state.played.contains_all(Cards::SCORING) {
            return state.won;
        }
        let seat = state.next_actor.unwrap();
        let plays = state
            .legal_plays(hands[seat.idx()])
            .distinct_plays(state.played, state.current_trick);
        if self.canonical && plays.len() == 1 {
            let mut child = state.clone();
            child.apply(&GameEvent::Play {
                seat,
                card: plays.max(),
            });
            return self.solve(&child, hands);
        }
        let remaining = hands.map(|h| h - state.played);
        let (ownership, trick) = if self.canonical {
            canonical_key(state, &remaining)
        } else {
            (
                remaining.map(|h| h.bits),
                state
                    .current_trick
                    .into_iter()
                    .fold(0, |t, c| (t << 8) | (c as u64 + 1)),
            )
        };
        let key = Key {
            remaining: ownership,
            trick,
            won: state.won,
            next: seat,
            led: Suit::VALUES.iter().fold(0, |v, s| {
                v | ((state.led_suits.contains(*s) as u8) << s.idx())
            }),
        };
        if let Some(won) = self.cache.get(&key) {
            return *won;
        }
        let mut best = state.won;
        let mut best_money = i16::MIN;
        for card in plays {
            let mut child = state.clone();
            child.play_card(seat, card);
            let won = self.solve(&child, hands);
            let money = won.scores(state.charges).money(seat);
            if money > best_money {
                best = won;
                best_money = money;
            }
        }
        // A fixed memory ceiling still permits exact uncached traversal.
        if self.cache.len() < 65536 {
            self.cache.insert(key, best);
        }
        best
    }
}

/// Neutral ranks can be renamed without changing future choices/payoffs.
/// Keep ownership order, nines, and the chargeable scoring card in each suit.
/// Won points and remaining scoring cards also determine the current trick's
/// scoring content, so its summary needs only control and the extended-nine rule.
fn canonical_key(state: &GameState, remaining: &[Cards; 4]) -> ([u64; 4], u64) {
    let unplayed = Cards::ALL - state.played;
    let ownership = Suit::VALUES.map(|suit| {
        let mut key = 1u64;
        for card in unplayed & suit.cards() {
            let owner = remaining.iter().position(|h| h.contains(card)).unwrap();
            let kind = if Cards::CHARGEABLE.contains(card) {
                2
            } else if card.rank() == Rank::Nine {
                1
            } else {
                0
            };
            key = (key << 4) | (kind << 2) | owner as u64;
        }
        key
    });
    let trick = state.current_trick;
    let control = if trick.is_empty() {
        0
    } else {
        let suit = trick.suit();
        let cards = trick.cards();
        let above = (unplayed & suit.cards())
            .above((cards & suit.cards()).max())
            .len();
        ((trick.len() as u64) << 9)
            | ((cards.contains(suit.with_rank(Rank::Nine)) as u64) << 8)
            | ((above as u64) << 4)
            | ((suit.idx() as u64) << 2)
            | trick.winning_seat(state.next_actor.unwrap()).idx() as u64
    };
    (ownership, control)
}
