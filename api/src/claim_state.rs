use crate::Seat;

#[derive(Copy, Clone, Debug)]
pub struct ClaimState {
    accepts: u16,
}

impl ClaimState {
    pub fn new() -> Self {
        Self { accepts: 0 }
    }

    pub fn successfully_claimed(self, claimer: Seat) -> bool {
        let accepts = self.accepts >> (4 * claimer.idx());
        accepts & 0xf == 0xf
    }

    pub fn is_claiming(self, seat: Seat) -> bool {
        let accepts = self.accepts >> (4 * seat.idx());
        accepts & 0xf != 0
    }

    pub fn has_accepted(self, claimer: Seat, acceptor: Seat) -> bool {
        let idx = 4 * claimer.idx() + acceptor.idx();
        self.accepts & (1 << idx) != 0
    }

    #[must_use]
    pub fn claim(self, seat: Seat) -> Self {
        self.accept(seat, seat)
    }

    #[must_use]
    pub fn accept(self, claimer: Seat, acceptor: Seat) -> Self {
        let idx = 4 * claimer.idx() + acceptor.idx();
        Self {
            accepts: self.accepts | (1 << idx),
        }
    }

    #[must_use]
    pub fn reject(self, claimer: Seat) -> Self {
        let mask = 0xf << (4 * claimer.idx());
        Self {
            accepts: self.accepts & !mask,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_claim() {
        let state = ClaimState::new();
        for &s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(s1));
            assert!(!state.is_claiming(s1));
            for &s2 in &Seat::VALUES {
                assert!(!state.has_accepted(s1, s2));
            }
        }
    }

    #[test]
    fn claim() {
        let state = ClaimState::new().claim(Seat::East);
        for &s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(s1));
            if s1 == Seat::East {
                assert!(state.is_claiming(s1));
            } else {
                assert!(!state.is_claiming(s1));
            }
            for &s2 in &Seat::VALUES {
                if s1 == Seat::East && s2 == Seat::East {
                    assert!(state.has_accepted(s1, s2));
                } else {
                    assert!(!state.has_accepted(s1, s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_once() {
        let state = ClaimState::new()
            .claim(Seat::East)
            .accept(Seat::East, Seat::North);
        for &s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(s1));
            if s1 == Seat::East {
                assert!(state.is_claiming(s1));
            } else {
                assert!(!state.is_claiming(s1));
            }
            for &s2 in &Seat::VALUES {
                if s1 == Seat::East && (s2 == Seat::East || s2 == Seat::North) {
                    assert!(state.has_accepted(s1, s2));
                } else {
                    assert!(!state.has_accepted(s1, s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_twice() {
        let state = ClaimState::new()
            .claim(Seat::East)
            .accept(Seat::East, Seat::North)
            .accept(Seat::East, Seat::South);
        for &s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(s1));
            if s1 == Seat::East {
                assert!(state.is_claiming(s1));
            } else {
                assert!(!state.is_claiming(s1));
            }
            for &s2 in &Seat::VALUES {
                if s1 == Seat::East && s2 != Seat::West {
                    assert!(state.has_accepted(s1, s2));
                } else {
                    assert!(!state.has_accepted(s1, s2));
                }
            }
        }
    }

    #[test]
    fn accept_claim_all() {
        let state = ClaimState::new()
            .claim(Seat::East)
            .accept(Seat::East, Seat::North)
            .accept(Seat::East, Seat::South)
            .accept(Seat::East, Seat::West);
        for &s1 in &Seat::VALUES {
            if s1 == Seat::East {
                assert!(state.successfully_claimed(s1));
            } else {
                assert!(!state.successfully_claimed(s1));
            }
            if s1 == Seat::East {
                assert!(state.is_claiming(s1));
            } else {
                assert!(!state.is_claiming(s1));
            }
            for &s2 in &Seat::VALUES {
                if s1 == Seat::East {
                    assert!(state.has_accepted(s1, s2));
                } else {
                    assert!(!state.has_accepted(s1, s2));
                }
            }
        }
    }

    #[test]
    fn reject_claim() {
        let state = ClaimState::new()
            .claim(Seat::East)
            .accept(Seat::East, Seat::North)
            .reject(Seat::East);
        for &s1 in &Seat::VALUES {
            assert!(!state.successfully_claimed(s1));
            assert!(!state.is_claiming(s1));
            for &s2 in &Seat::VALUES {
                assert!(!state.has_accepted(s1, s2));
            }
        }
    }
}
