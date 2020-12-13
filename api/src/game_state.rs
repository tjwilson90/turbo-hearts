use crate::{
    Card, Cards, ChargeState, ChargingRules, ClaimState, DoneState, GameEvent, GamePhase, Scores,
    Seat, Suits, Trick, WonState,
};

#[derive(Clone, Debug)]
pub struct GameState {
    pub rules: ChargingRules,     // 1
    pub phase: GamePhase,         // 1
    pub done: DoneState,          // 1
    pub charge_count: u8,         // 1
    pub charges: ChargeState,     // 2
    pub next_actor: Option<Seat>, // 1
    pub played: Cards,            // 8
    pub claims: ClaimState,       // 2
    pub won: WonState,            // 4
    pub led_suits: Suits,         // 1
    pub current_trick: Trick,     // 8
}

impl GameState {
    pub fn new() -> Self {
        Self {
            rules: ChargingRules::Classic,
            phase: GamePhase::PassLeft,
            done: DoneState::new(),
            charge_count: 0,
            charges: ChargeState::new(),
            next_actor: None,
            played: Cards::NONE,
            claims: ClaimState::new(),
            won: WonState::new(),
            led_suits: Suits::NONE,
            current_trick: Trick::new(),
        }
    }

    pub fn pass_status_event(&self) -> GameEvent {
        GameEvent::PassStatus {
            north_done: self.done.sent_pass(Seat::North),
            east_done: self.done.sent_pass(Seat::East),
            south_done: self.done.sent_pass(Seat::South),
            west_done: self.done.sent_pass(Seat::West),
        }
    }

    pub fn charge_status_event(&self) -> GameEvent {
        GameEvent::ChargeStatus {
            next_charger: self.next_actor,
            north_done: self.done.charged(Seat::North),
            east_done: self.done.charged(Seat::East),
            south_done: self.done.charged(Seat::South),
            west_done: self.done.charged(Seat::West),
        }
    }

    pub fn scores(&self) -> Scores {
        self.won.scores(self.charges)
    }

    pub fn can_charge(&self, seat: Seat) -> bool {
        match self.next_actor {
            Some(s) if s != seat => false,
            _ => true,
        }
    }

    pub fn apply(&mut self, event: &GameEvent) {
        match event {
            GameEvent::Sit { rules, .. } => {
                self.rules = *rules;
            }
            GameEvent::Deal { .. } => {
                self.charge_count = 0;
                self.charges = ChargeState::new();
                self.next_actor = self.phase.first_charger(self.rules);
                self.played = Cards::NONE;
                self.claims = ClaimState::new();
                self.won = WonState::new();
                self.led_suits = Suits::NONE;
                self.current_trick = Trick::new();
            }
            GameEvent::SendPass { from, .. } => {
                self.done = self.done.send_pass(*from);
            }
            GameEvent::RecvPass { to, .. } => {
                self.done = self.done.recv_pass(*to);
                if self.done.all_recv_pass() {
                    self.phase = self.phase.next(self.charge_count != 0);
                    self.done = DoneState::new();
                    self.next_actor = self.phase.first_charger(self.rules);
                }
            }
            GameEvent::BlindCharge { seat, count } => {
                self.charge_count += *count as u8;
                self.charge(*seat, *count);
            }
            GameEvent::Charge { seat, cards } => {
                self.charge_count += cards.len() as u8;
                self.charges = self.charges.charge(*seat, *cards);
                self.charge(*seat, cards.len());
            }
            GameEvent::RevealCharges {
                north,
                east,
                south,
                west,
            } => {
                self.charges = self
                    .charges
                    .charge(Seat::North, *north)
                    .charge(Seat::East, *east)
                    .charge(Seat::South, *south)
                    .charge(Seat::West, *west);
            }
            GameEvent::Play { seat, card } => {
                self.played |= *card;
                self.current_trick = self.current_trick.push(*card);
                self.next_actor = Some(seat.left());
                if self.current_trick.is_complete() || self.played == Cards::ALL {
                    self.led_suits |= self.current_trick.suit();
                    let winning_seat = self.current_trick.winning_seat(seat.left());
                    self.won = self.won.win(winning_seat, self.current_trick.cards());
                    self.current_trick = Trick::new();
                    self.next_actor = Some(winning_seat);
                    if self.played == Cards::ALL {
                        self.phase = self.phase.next(self.charge_count != 0);
                        self.done = DoneState::new();
                    }
                }
            }
            GameEvent::Claim { seat, .. } => {
                self.claims = self.claims.claim(*seat);
            }
            GameEvent::AcceptClaim { claimer, acceptor } => {
                self.claims = self.claims.accept(*claimer, *acceptor);
                if self.claims.successfully_claimed(*claimer) {
                    self.won = self.won.win(
                        *claimer,
                        (Cards::ALL - self.played) | self.current_trick.cards(),
                    );
                    self.current_trick = Trick::new();
                    self.phase = self.phase.next(self.charge_count != 0);
                    self.done = DoneState::new();
                    self.next_actor = None;
                }
            }
            GameEvent::RejectClaim { claimer, .. } => {
                self.claims = self.claims.reject(*claimer);
            }
            _ => {}
        }
    }

    fn charge(&mut self, seat: Seat, count: usize) {
        if let Some(charger) = &mut self.next_actor {
            *charger = charger.left();
        }
        if count == 0 {
            self.done = self.done.charge(seat);
            if self.done.all_charge() {
                self.phase = self.phase.next(self.charge_count != 0);
                self.done = DoneState::new();
                self.next_actor = None;
            }
        } else {
            self.done = DoneState::new();
            if !self.rules.chain() {
                self.done = self.done.charge(seat);
            }
        }
    }

    pub fn legal_plays(&self, cards: Cards) -> Cards {
        let mut plays = cards - self.played;

        // if you have the two of clubs
        if plays.contains(Card::TwoClubs) {
            // you must play it
            return Card::TwoClubs.into();
        }

        // if this is the first trick
        if self.current_trick.cards().contains(Card::TwoClubs) {
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
            let suit = self.current_trick.suit();

            // and you have any cards in suit
            if suit.cards().contains_any(plays) {
                // you must play in suit
                plays &= suit.cards();

                // and if this is the first trick of this suit
                if !self.led_suits.contains(suit)
                    // and you have multiple plays
                    && plays.len() > 1
                {
                    // you cannot play charged cards from the suit
                    plays -= self.charges.all_charges();
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

            let unled_charges = self.charges.all_charges() - self.led_suits.cards();
            // if you have cards other than charged cards from unled suits
            if !unled_charges.contains_all(plays) {
                // you must lead one of them
                plays -= unled_charges;
            }
        }
        plays
    }
}
