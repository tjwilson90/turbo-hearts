use crate::{
    card::Card,
    cards::Cards,
    game::{claim::ClaimState, event::GameEvent, phase::GamePhase},
    rank::Rank,
    seat::Seat,
    types::ChargingRules,
    user::UserId,
};

#[derive(Debug)]
pub struct GameState {
    pub players: [UserId; 4],
    pub rules: ChargingRules,
    pub phase: GamePhase,
    pub sent_pass: [bool; 4],
    pub received_pass: [bool; 4],
    pub charge_count: usize,
    pub charged: [Cards; 4],
    pub done_charging: [bool; 4],
    pub next_charger: Option<Seat>,
    pub played: Cards,
    pub claims: ClaimState,
    pub won: [Cards; 4],
    pub led_suits: Cards,
    pub next_player: Option<Seat>,
    pub current_trick: Vec<Card>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: [UserId::null(); 4],
            rules: ChargingRules::Classic,
            phase: GamePhase::PassLeft,
            sent_pass: [false; 4],
            received_pass: [false; 4],
            charge_count: 0,
            charged: [Cards::NONE; 4],
            done_charging: [false; 4],
            next_charger: None,
            played: Cards::NONE,
            claims: ClaimState::new(),
            won: [Cards::NONE; 4],
            led_suits: Cards::NONE,
            next_player: None,
            current_trick: Vec::with_capacity(8),
        }
    }

    pub fn charged_cards(&self) -> Cards {
        self.charged.iter().cloned().collect()
    }

    pub fn pass_status_event(&self) -> GameEvent {
        GameEvent::PassStatus {
            north_done: self.sent_pass[Seat::North.idx()],
            east_done: self.sent_pass[Seat::East.idx()],
            south_done: self.sent_pass[Seat::South.idx()],
            west_done: self.sent_pass[Seat::West.idx()],
        }
    }

    pub fn charge_status_event(&self) -> GameEvent {
        GameEvent::ChargeStatus {
            next_charger: self.next_charger,
            north_done: self.done_charging[Seat::North.idx()],
            east_done: self.done_charging[Seat::East.idx()],
            south_done: self.done_charging[Seat::South.idx()],
            west_done: self.done_charging[Seat::West.idx()],
        }
    }

    pub fn score(&self, seat: Seat) -> i16 {
        let charged = self.charged_cards();
        let won = self.won[seat.idx()];
        let hearts = match (
            (won & Cards::HEARTS).len() as i16,
            charged.contains(Card::AceHearts),
        ) {
            (hearts, true) => 2 * hearts,
            (hearts, _) => hearts,
        };
        let queen = match (
            won.contains(Card::QueenSpades),
            charged.contains(Card::QueenSpades),
        ) {
            (true, true) => 26,
            (true, false) => 13,
            _ => 0,
        };
        let jack = match (
            won.contains(Card::JackDiamonds),
            charged.contains(Card::JackDiamonds),
        ) {
            (true, true) => -20,
            (true, false) => -10,
            _ => 0,
        };
        let ten = match (
            won.contains(Card::TenClubs),
            charged.contains(Card::TenClubs),
        ) {
            (true, true) => 4,
            (true, false) => 2,
            _ => 1,
        };
        if won.contains(Card::QueenSpades) && won.contains_all(Cards::HEARTS) {
            ten * (jack - hearts - queen)
        } else {
            ten * (jack + hearts + queen)
        }
    }

    pub fn can_charge(&self, seat: Seat) -> bool {
        match self.next_charger {
            Some(s) if s != seat => false,
            _ => true,
        }
    }

    pub fn apply(&mut self, event: &GameEvent) {
        match event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
                ..
            } => {
                self.players[0] = north.user_id();
                self.players[1] = east.user_id();
                self.players[2] = south.user_id();
                self.players[3] = west.user_id();
                self.rules = *rules;
            }
            GameEvent::Deal { .. } => {
                self.charge_count = 0;
                self.charged = [Cards::NONE; 4];
                self.done_charging = [false; 4];
                self.next_charger = self.phase.first_charger(self.rules);
                self.sent_pass = [false; 4];
                self.received_pass = [false; 4];
                self.next_player = None;
                self.played = Cards::NONE;
                self.claims = ClaimState::new();
                self.won = [Cards::NONE; 4];
                self.led_suits = Cards::NONE;
                self.current_trick.clear();
            }
            GameEvent::SendPass { from, .. } => {
                self.sent_pass[from.idx()] = true;
            }
            GameEvent::RecvPass { to, .. } => {
                self.received_pass[to.idx()] = true;
                if self.received_pass.iter().all(|b| *b) {
                    self.phase = self.phase.next(self.charge_count != 0);
                    self.done_charging = [false, false, false, false];
                    self.next_charger = self.phase.first_charger(self.rules);
                }
            }
            GameEvent::BlindCharge { seat, count } => {
                self.charge_count += *count;
                self.charge(*seat, *count);
            }
            GameEvent::Charge { seat, cards } => {
                self.charge_count += cards.len();
                self.charged[seat.idx()] |= *cards;
                self.charge(*seat, cards.len());
            }
            GameEvent::RevealCharges {
                north,
                east,
                south,
                west,
            } => {
                self.charged[0] = *north;
                self.charged[1] = *east;
                self.charged[2] = *south;
                self.charged[3] = *west;
            }
            GameEvent::Play { seat, card } => {
                self.played |= *card;
                self.current_trick.push(*card);
                self.next_player = Some(seat.left());
                if self.current_trick.len() == 8
                    || self.played == Cards::ALL
                    || (self.current_trick.len() == 4
                        && !self
                            .current_trick
                            .contains(&self.current_trick[0].with_rank(Rank::Nine)))
                {
                    self.led_suits |= self.current_trick[0].suit().cards();
                    let mut seat = seat.left();
                    let mut winning_seat = seat;
                    let mut winning_card = self.current_trick[0];
                    for card in &self.current_trick[1..] {
                        seat = seat.left();
                        if card.suit() == winning_card.suit() && card.rank() > winning_card.rank() {
                            winning_card = *card;
                            winning_seat = seat;
                        }
                    }
                    self.won[winning_seat.idx()] |=
                        self.current_trick.iter().cloned().collect::<Cards>();
                    self.next_player = Some(winning_seat);
                    self.current_trick.clear();
                    if self.played == Cards::ALL {
                        self.phase = self.phase.next(self.charge_count != 0);
                    }
                }
            }
            GameEvent::Claim { seat, .. } => {
                self.claims.claim(*seat);
            }
            GameEvent::AcceptClaim { claimer, acceptor } => {
                if self.claims.accept(*claimer, *acceptor) {
                    self.won[claimer.idx()] |=
                        self.current_trick.iter().cloned().collect::<Cards>();
                    self.won[claimer.idx()] |= Cards::ALL - self.played;
                    self.phase = self.phase.next(self.charge_count != 0);
                }
            }
            GameEvent::RejectClaim { claimer, .. } => {
                self.claims.reject(*claimer);
            }
            _ => {}
        }
    }

    fn charge(&mut self, seat: Seat, count: usize) {
        if let Some(charger) = &mut self.next_charger {
            *charger = charger.left();
        }
        if count == 0 {
            self.done_charging[seat.idx()] = true;
            if self.done_charging.iter().all(|b| *b) {
                self.phase = self.phase.next(self.charge_count != 0);
            }
        } else {
            self.done_charging.iter_mut().for_each(|b| *b = false);
            self.done_charging[seat.idx()] = !self.rules.chain();
        }
    }

    pub fn legal_plays(&self, cards: Cards) -> Cards {
        let mut plays = cards - self.played;
        // if this is the first trick
        if self.current_trick.len() == self.played.len() {
            // if you have the two of clubs
            if plays.contains(Card::TwoClubs) {
                // you must play it
                return Card::TwoClubs.into();
            }

            // if you have a non-point card
            if !Cards::POINTS.contains_all(plays) {
                // you cannot play points
                plays -= Cards::POINTS;

            // otherwise, if you have the jack of diamonds
            } else if plays.contains(Card::JackDiamonds) {
                // you must play it
                return Card::JackDiamonds.into();

            // otherwise, if you have the queen of spades
            } else if plays.contains(Card::QueenSpades) {
                // you must play it
                return Card::QueenSpades.into();
            }
        }

        // if this is not the first card in the trick
        if !self.current_trick.is_empty() {
            let suit = self.current_trick[0].suit();

            // and you have any cards in suit
            if suit.cards().contains_any(plays) {
                // you must play in suit
                plays &= suit.cards();

                // and if this is the first trick of this suit
                if !self.led_suits.contains_any(suit.cards())
                    // and you have multiple plays
                    && plays.len() > 1
                {
                    // you cannot play charged cards from the suit
                    plays -= self.charged_cards();
                }
            }

        // otherwise, you are leading the trick
        } else {
            // If hearts are not broken
            if !self.played.contains_any(Cards::HEARTS)
                // and you have a non-heart
                && !Cards::HEARTS.contains_all(plays)
            {
                // you cannot lead hearts
                plays -= Cards::HEARTS;
            }

            let unled_charges = self.charged_cards() - self.led_suits;
            // if you have cards other than charged cards from unled suits
            if !unled_charges.contains_all(plays) {
                // you must lead one of them
                plays -= unled_charges;
            }
        }
        plays
    }
}
