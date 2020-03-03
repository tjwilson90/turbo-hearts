use crate::seat::Seat;

#[derive(Debug)]
pub struct ClaimState {
    accepts: [[bool; 4]; 4],
}

impl ClaimState {
    pub fn new() -> Self {
        Self {
            accepts: [[false; 4]; 4],
        }
    }

    pub fn is_claiming(&self, seat: Seat) -> bool {
        self.accepts[seat.idx()][seat.idx()]
    }

    pub fn has_accepted(&self, claimer: Seat, acceptor: Seat) -> bool {
        self.accepts[claimer.idx()][acceptor.idx()]
    }

    pub fn claim(&mut self, seat: Seat) {
        self.accepts[seat.idx()][seat.idx()] = true;
    }

    pub fn accept(&mut self, claimer: Seat, acceptor: Seat) -> bool {
        self.accepts[claimer.idx()][acceptor.idx()] = true;
        self.accepts[claimer.idx()].iter().all(|b| *b)
    }

    pub fn reject(&mut self, claimer: Seat) {
        self.accepts[claimer.idx()]
            .iter_mut()
            .for_each(|b| *b = false);
    }
}
