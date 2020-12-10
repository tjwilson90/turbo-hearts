use crate::Seat;

#[derive(Clone, Copy, Debug)]
pub struct Scores {
    scores: [i16; 4],
}

impl Scores {
    pub fn new(scores: [i16; 4]) -> Self {
        Self { scores }
    }

    pub fn score(self, seat: Seat) -> i16 {
        self.scores[seat.idx()]
    }

    pub fn money(self, seat: Seat) -> i16 {
        self.scores[0] + self.scores[1] + self.scores[2] + self.scores[3]
            - 4 * self.scores[seat.idx()]
    }
}
