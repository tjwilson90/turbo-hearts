use crate::Seat;

#[derive(Copy, Clone, Debug)]
pub struct ClaimState {
    accepts: u16,
}

impl ClaimState {
    pub fn new() -> Self {
        Self { accepts: 0 }
    }

    pub fn will_successfully_claim(&self, claimer: Seat, acceptor: Seat) -> bool {
        let mut accepts = self.accepts >> (4 * claimer.idx());
        accepts |= 1 << acceptor.idx();
        accepts & 0xf == 0xf
    }

    pub fn successfully_claimed(&self, claimer: Seat) -> bool {
        let accepts = self.accepts >> (4 * claimer.idx());
        accepts & 0xf == 0xf
    }

    pub fn is_claiming(&self, seat: Seat) -> bool {
        let accepts = self.accepts >> (4 * seat.idx());
        accepts & 0xf != 0
    }

    pub fn has_accepted(&self, claimer: Seat, acceptor: Seat) -> bool {
        let idx = 4 * claimer.idx() + acceptor.idx();
        self.accepts & (1 << idx) != 0
    }

    pub fn claim(&mut self, seat: Seat) {
        let idx = 5 * seat.idx();
        self.accepts |= 1 << idx;
    }

    pub fn accept(&mut self, claimer: Seat, acceptor: Seat) -> bool {
        let idx = 4 * claimer.idx() + acceptor.idx();
        self.accepts |= 1 << idx;
        self.successfully_claimed(claimer)
    }

    pub fn reject(&mut self, claimer: Seat) {
        let mask = 0xf << (4 * claimer.idx());
        self.accepts &= !mask;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_claim() {
        let state = ClaimState::new();
        for s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(*s1));
            assert!(!state.is_claiming(*s1));
            for s2 in &Seat::VALUES {
                assert!(!state.will_successfully_claim(*s1, *s2));
                assert!(!state.has_accepted(*s1, *s2));
            }
        }
    }

    #[test]
    fn claim() {
        let mut state = ClaimState::new();
        state.claim(Seat::East);
        for s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(*s1));
            if *s1 == Seat::East {
                assert!(state.is_claiming(*s1));
            } else {
                assert!(!state.is_claiming(*s1));
            }
            for s2 in &Seat::VALUES {
                assert!(!state.will_successfully_claim(*s1, *s2));
                if *s1 == Seat::East && *s2 == Seat::East {
                    assert!(state.has_accepted(*s1, *s2));
                } else {
                    assert!(!state.has_accepted(*s1, *s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_once() {
        let mut state = ClaimState::new();
        state.claim(Seat::East);
        state.accept(Seat::East, Seat::North);
        for s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(*s1));
            if *s1 == Seat::East {
                assert!(state.is_claiming(*s1));
            } else {
                assert!(!state.is_claiming(*s1));
            }
            for s2 in &Seat::VALUES {
                assert!(!state.will_successfully_claim(*s1, *s2));
                if *s1 == Seat::East && (*s2 == Seat::East || *s2 == Seat::North) {
                    assert!(state.has_accepted(*s1, *s2));
                } else {
                    assert!(!state.has_accepted(*s1, *s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_twice() {
        let mut state = ClaimState::new();
        state.claim(Seat::East);
        state.accept(Seat::East, Seat::North);
        state.accept(Seat::East, Seat::South);
        for s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(*s1));
            if *s1 == Seat::East {
                assert!(state.is_claiming(*s1));
            } else {
                assert!(!state.is_claiming(*s1));
            }
            for s2 in &Seat::VALUES {
                if *s1 == Seat::East && *s2 == Seat::West {
                    assert!(state.will_successfully_claim(*s1, *s2));
                } else {
                    assert!(!state.will_successfully_claim(*s1, *s2));
                }
                if *s1 == Seat::East && *s2 != Seat::West {
                    assert!(state.has_accepted(*s1, *s2));
                } else {
                    assert!(!state.has_accepted(*s1, *s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_all() {
        let mut state = ClaimState::new();
        state.claim(Seat::East);
        state.accept(Seat::East, Seat::North);
        state.accept(Seat::East, Seat::South);
        state.accept(Seat::East, Seat::West);
        for s1 in &Seat::VALUES {
            if *s1 == Seat::East {
                assert!(state.successfully_claimed(*s1));
            } else {
                assert!(!state.successfully_claimed(*s1));
            }
            if *s1 == Seat::East {
                assert!(state.is_claiming(*s1));
            } else {
                assert!(!state.is_claiming(*s1));
            }
            for s2 in &Seat::VALUES {
                if *s1 == Seat::East {
                    assert!(state.will_successfully_claim(*s1, *s2));
                } else {
                    assert!(!state.will_successfully_claim(*s1, *s2));
                }
                if *s1 == Seat::East {
                    assert!(state.has_accepted(*s1, *s2));
                } else {
                    assert!(!state.has_accepted(*s1, *s2));
                }
            }
        }
    }

    #[test]
    fn reject_claim() {
        let mut state = ClaimState::new();
        state.claim(Seat::East);
        state.accept(Seat::East, Seat::North);
        state.reject(Seat::East);
        for s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(*s1));
            assert!(!state.is_claiming(*s1));
            for s2 in &Seat::VALUES {
                assert!(!state.will_successfully_claim(*s1, *s2));
                assert!(!state.has_accepted(*s1, *s2));
            }
        }
    }
}
