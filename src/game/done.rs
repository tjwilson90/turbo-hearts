use crate::seat::Seat;

#[derive(Debug)]
pub struct DoneState {
    state: u8,
}

impl DoneState {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn reset(&mut self) {
        self.state = 0;
    }

    pub fn send_pass(&mut self, seat: Seat) {
        self.state |= 1 << seat.idx();
    }

    pub fn sent_pass(&self, seat: Seat) -> bool {
        self.state & (1 << seat.idx()) != 0
    }

    pub fn recv_pass(&mut self, seat: Seat) {
        self.state |= 1 << (4 + seat.idx());
    }

    pub fn all_recv_pass(&self) -> bool {
        self.state & 0xf0 == 0xf0
    }

    pub fn charge(&mut self, seat: Seat) {
        self.state |= 1 << seat.idx();
    }

    pub fn charged(&self, seat: Seat) -> bool {
        self.state & (1 << seat.idx()) != 0
    }

    pub fn all_charge(&self) -> bool {
        self.state & 0x0f == 0x0f
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let state = DoneState::new();
        for seat in &Seat::VALUES {
            assert!(!state.sent_pass(*seat));
            assert!(!state.charged(*seat));
        }
        assert!(!state.all_recv_pass());
        assert!(!state.all_charge());
    }

    #[test]
    fn reset() {
        let mut state = DoneState::new();
        state.send_pass(Seat::North);
        state.reset();
        for seat in &Seat::VALUES {
            assert!(!state.sent_pass(*seat));
            assert!(!state.charged(*seat));
        }
        assert!(!state.all_recv_pass());
        assert!(!state.all_charge());
    }

    #[test]
    fn pass_one() {
        let mut state = DoneState::new();
        state.send_pass(Seat::North);
        assert!(state.sent_pass(Seat::North));
        assert!(!state.sent_pass(Seat::East));
        assert!(!state.sent_pass(Seat::South));
        assert!(!state.sent_pass(Seat::West));
    }

    #[test]
    fn send_pass_all() {
        let mut state = DoneState::new();
        state.send_pass(Seat::North);
        state.send_pass(Seat::East);
        state.send_pass(Seat::South);
        state.send_pass(Seat::West);
        assert!(state.sent_pass(Seat::North));
        assert!(state.sent_pass(Seat::East));
        assert!(state.sent_pass(Seat::South));
        assert!(state.sent_pass(Seat::West));
        assert!(!state.all_recv_pass());
    }

    #[test]
    fn recv_pass_all() {
        let mut state = DoneState::new();
        state.recv_pass(Seat::North);
        state.recv_pass(Seat::East);
        state.recv_pass(Seat::South);
        state.recv_pass(Seat::West);
        assert!(state.all_recv_pass());
    }

    #[test]
    fn charge_all() {
        let mut state = DoneState::new();
        state.charge(Seat::North);
        state.charge(Seat::East);
        state.charge(Seat::South);
        state.charge(Seat::West);
        assert!(state.charged(Seat::North));
        assert!(state.charged(Seat::East));
        assert!(state.charged(Seat::South));
        assert!(state.charged(Seat::West));
        assert!(state.all_charge());
    }
}
