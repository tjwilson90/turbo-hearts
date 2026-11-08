//! Opt-in likelihood of a human's first charging decision. Ownership/void
//! constraints remain exact; this only weights otherwise feasible deals.
use turbo_hearts_api::{Card, Cards, ChargingRules, GameEvent, GameState, PassDirection, Seat};

mod fitted {
    include!("../../assets/models/charge-policy.rs");
}

const CHARGES: [Card; 4] = [
    Card::QueenSpades,
    Card::AceHearts,
    Card::JackDiamonds,
    Card::TenClubs,
];

#[derive(Clone, Default)]
pub(crate) struct ChargeHistory {
    humans: [bool; 4],
    // Selected cards and cards already unavailable at the first opportunity.
    observations: [Option<(Cards, Cards)>; 4],
}

impl ChargeHistory {
    pub fn on_event(&mut self, state: &GameState, event: &GameEvent) {
        match event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                ..
            } => {
                self.humans = [north, east, south, west].map(|p| p.strategy().is_none());
            }
            GameEvent::Deal { .. } | GameEvent::HandComplete { .. } => {
                self.observations = [None; 4]
            }
            GameEvent::Charge { seat, cards }
                if self.humans[seat.idx()]
                    && state.rules == ChargingRules::Classic
                    && state.phase.direction() != PassDirection::Keeper
                    && self.observations[seat.idx()].is_none() =>
            {
                self.observations[seat.idx()] = Some((*cards, state.charges.all_charges()));
            }
            _ => {}
        }
    }

    pub fn has_evidence(&self, root: Seat) -> bool {
        Seat::VALUES
            .iter()
            .any(|&s| s != root && self.observations[s.idx()].is_some())
    }

    pub fn log_likelihood(&self, hands: [Cards; 4], root: Seat) -> f64 {
        let mut result = 0.0;
        for seat in Seat::VALUES {
            if seat == root {
                continue;
            }
            if let Some((selected, unavailable)) = self.observations[seat.idx()] {
                let hand = hands[seat.idx()];
                for (index, card) in CHARGES.iter().enumerate() {
                    if hand.contains(*card) && !unavailable.contains(*card) {
                        let length = (hand & card.suit().cards()).len();
                        let bin = length.saturating_sub(3).min(4);
                        let p = fitted::PROBABILITIES[index][bin];
                        result += if selected.contains(*card) {
                            p.ln()
                        } else {
                            (-p).ln_1p()
                        };
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_hearts_api::GamePhase;

    #[test]
    fn every_charged_suit_favors_length_but_a_decline_is_negative_evidence() {
        let mut state = GameState::new();
        state.phase = GamePhase::ChargeLeft;
        for card in CHARGES {
            let mut history = ChargeHistory {
                humans: [true; 4],
                ..Default::default()
            };
            history.on_event(
                &state,
                &GameEvent::Charge {
                    seat: Seat::East,
                    cards: card.into(),
                },
            );
            let mut short = [Cards::NONE; 4];
            short[1] = card.into();
            let mut long = short;
            long[1] |= (card.suit().cards() - card)
                .into_iter()
                .take(6)
                .collect::<Cards>();
            assert!(
                history.log_likelihood(long, Seat::North)
                    > history.log_likelihood(short, Seat::North)
            );
            let once = history.log_likelihood(long, Seat::North);
            history.on_event(
                &state,
                &GameEvent::Charge {
                    seat: Seat::East,
                    cards: Cards::NONE,
                },
            );
            assert_eq!(history.log_likelihood(long, Seat::North), once);
            assert_eq!(history.log_likelihood(long, Seat::East), 0.0);
            history.observations[1] = Some((Cards::NONE, Cards::NONE));
            assert!(
                history.log_likelihood(long, Seat::North)
                    < history.log_likelihood(short, Seat::North)
            );
        }
    }

    #[test]
    fn excludes_bots_blind_rules_and_pre_pass_keeper_charging() {
        let event = GameEvent::Charge {
            seat: Seat::East,
            cards: Card::QueenSpades.into(),
        };
        let mut state = GameState::new();
        let mut history = ChargeHistory::default();
        history.on_event(&state, &event);
        assert!(!history.has_evidence(Seat::North));
        history.humans = [true; 4];
        state.rules = ChargingRules::Blind;
        history.on_event(&state, &event);
        assert!(!history.has_evidence(Seat::North));
        state.rules = ChargingRules::Classic;
        state.phase = GamePhase::ChargeKeeper1;
        history.on_event(&state, &event);
        assert!(!history.has_evidence(Seat::North));
        state.phase = GamePhase::ChargeLeft;
        history.on_event(&state, &event);
        assert!(history.has_evidence(Seat::North));
    }
}
