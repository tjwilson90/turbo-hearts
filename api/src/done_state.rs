use crate::Seat;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DoneState {
    state: u8,
}

impl DoneState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub fn send_pass(self, seat: Seat) -> Self {
        Self {
            state: self.state | (1 << seat.idx()),
        }
    }

    pub fn sent_pass(self, seat: Seat) -> bool {
        self.state & (1 << seat.idx()) != 0
    }

    #[must_use]
    pub fn recv_pass(self, seat: Seat) -> Self {
        Self {
            state: self.state | (1 << (4 + seat.idx())),
        }
    }

    pub fn all_recv_pass(self) -> bool {
        self.state & 0xf0 == 0xf0
    }

    #[must_use]
    pub fn charge(self, seat: Seat) -> Self {
        Self {
            state: self.state | (1 << seat.idx()),
        }
    }

    pub fn charged(self, seat: Seat) -> bool {
        self.state & (1 << seat.idx()) != 0
    }

    pub fn all_charge(self) -> bool {
        self.state & 0x0f == 0x0f
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let state = DoneState::new();
        for &seat in &Seat::VALUES {
            assert!(!state.sent_pass(seat));
            assert!(!state.charged(seat));
        }
        assert!(!state.all_recv_pass());
        assert!(!state.all_charge());
    }

    #[test]
    fn pass_one() {
        let state = DoneState::new().send_pass(Seat::North);
        assert!(state.sent_pass(Seat::North));
        assert!(!state.sent_pass(Seat::East));
        assert!(!state.sent_pass(Seat::South));
        assert!(!state.sent_pass(Seat::West));
    }

    #[test]
    fn send_pass_all() {
        let state = DoneState::new()
            .send_pass(Seat::North)
            .send_pass(Seat::East)
            .send_pass(Seat::South)
            .send_pass(Seat::West);
        assert!(state.sent_pass(Seat::North));
        assert!(state.sent_pass(Seat::East));
        assert!(state.sent_pass(Seat::South));
        assert!(state.sent_pass(Seat::West));
        assert!(!state.all_recv_pass());
    }

    #[test]
    fn recv_pass_all() {
        let state = DoneState::new()
            .recv_pass(Seat::North)
            .recv_pass(Seat::East)
            .recv_pass(Seat::South)
            .recv_pass(Seat::West);
        assert!(state.all_recv_pass());
    }

    #[test]
    fn charge_all() {
        let state = DoneState::new()
            .charge(Seat::North)
            .charge(Seat::East)
            .charge(Seat::South)
            .charge(Seat::West);
        assert!(state.charged(Seat::North));
        assert!(state.charged(Seat::East));
        assert!(state.charged(Seat::South));
        assert!(state.charged(Seat::West));
        assert!(state.all_charge());
    }
}
