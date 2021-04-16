use crate::{
    Card, Cards, GameEvent, GameId, GamePhase, GameState, HashedSeed, PassDirection, RulesError,
    Seat, UserId,
};

#[derive(Clone, Debug)]
pub struct Game<S> {
    pub events: Vec<GameEvent>,
    pub subscribers: Vec<(UserId, S)>,
    pub bots: Vec<(Seat, S)>,
    pub pre_pass_hand: [Cards; 4],
    pub post_pass_hand: [Cards; 4],
    pub players: [UserId; 4],
    pub state: GameState,
    pub seed: HashedSeed,
}

impl<S> Game<S> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            subscribers: Vec::new(),
            bots: Vec::new(),
            pre_pass_hand: [Cards::NONE; 4],
            post_pass_hand: [Cards::NONE; 4],
            players: [UserId::null(); 4],
            state: GameState::new(),
            seed: HashedSeed::new(),
        }
    }

    fn owner(&self, card: Card) -> Seat {
        for &seat in &Seat::VALUES {
            if self.post_pass_hand[seat.idx()].contains(card) {
                return seat;
            }
        }
        unreachable!()
    }

    pub fn seat(&self, user_id: UserId) -> Option<Seat> {
        self.players
            .iter()
            .position(|&id| id == user_id)
            .map(|idx| Seat::VALUES[idx])
    }

    fn play_status_event(&self, leader: Seat) -> GameEvent {
        GameEvent::PlayStatus {
            next_player: leader,
            legal_plays: self
                .state
                .legal_plays(self.post_pass_hand[leader.idx()] - self.state.played),
        }
    }

    pub fn apply<F>(&mut self, event: &GameEvent, mut broadcast: F)
    where
        F: FnMut(&mut Game<S>, &GameEvent),
    {
        broadcast(self, &event);
        self.state.apply(&event);
        self.events.push(event.clone());
        match &event {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                seed,
                ..
            } => {
                self.players[0] = north.user_id();
                self.players[1] = east.user_id();
                self.players[2] = south.user_id();
                self.players[3] = west.user_id();
                self.seed = seed.into();
            }
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                ..
            } => {
                debug_assert_eq!(Cards::ALL, *north | *east | *south | *west);
                self.pre_pass_hand[Seat::North.idx()] = *north;
                self.post_pass_hand[Seat::North.idx()] = *north;
                self.pre_pass_hand[Seat::East.idx()] = *east;
                self.post_pass_hand[Seat::East.idx()] = *east;
                self.pre_pass_hand[Seat::South.idx()] = *south;
                self.post_pass_hand[Seat::South.idx()] = *south;
                self.pre_pass_hand[Seat::West.idx()] = *west;
                self.post_pass_hand[Seat::West.idx()] = *west;
                if self.state.phase.is_passing() {
                    broadcast(self, &GameEvent::StartPassing);
                    broadcast(self, &self.state.pass_status_event());
                } else {
                    broadcast(self, &GameEvent::StartCharging);
                    broadcast(self, &self.state.charge_status_event());
                }
            }
            GameEvent::SendPass { from, cards } => {
                debug_assert_eq!(cards.len(), 3);
                debug_assert!(self.post_pass_hand[from.idx()].contains_all(*cards));
                self.post_pass_hand[from.idx()] -= *cards;
                broadcast(self, &self.state.pass_status_event());
            }
            GameEvent::RecvPass { to, cards } => {
                debug_assert_eq!(cards.len(), 3);
                debug_assert!(!self.post_pass_hand[to.idx()].contains_any(*cards));
                self.post_pass_hand[to.idx()] |= *cards;
                if self.state.phase.is_charging() {
                    broadcast(self, &GameEvent::StartCharging);
                    broadcast(self, &self.state.charge_status_event());
                }
            }
            GameEvent::Charge { seat, cards } => {
                debug_assert!(self.post_pass_hand[seat.idx()].contains_all(*cards));
                broadcast(self, &self.state.charge_status_event());
                if self.state.phase.is_passing() {
                    broadcast(self, &GameEvent::StartPassing);
                    broadcast(self, &self.state.pass_status_event());
                } else if self.state.phase.is_playing() {
                    let leader = self.owner(Card::TwoClubs);
                    self.state.next_actor = Some(leader);
                    if self.state.rules.blind() {
                        let charges = self.state.charges.all_charges();
                        let reveal = GameEvent::RevealCharges {
                            north: self.post_pass_hand[0] & charges,
                            east: self.post_pass_hand[1] & charges,
                            south: self.post_pass_hand[2] & charges,
                            west: self.post_pass_hand[3] & charges,
                        };
                        broadcast(self, &reveal);
                        self.state.apply(&reveal);
                    }
                    broadcast(self, &GameEvent::StartTrick { leader });
                    broadcast(self, &self.play_status_event(leader));
                }
            }
            GameEvent::Play { seat, card } => {
                debug_assert!(self.post_pass_hand[seat.idx()].contains(*card));
                if self.state.current_trick.is_empty() {
                    let winner = self.state.next_actor.unwrap();
                    broadcast(self, &GameEvent::EndTrick { winner });
                    if self.state.phase.is_playing() {
                        broadcast(self, &GameEvent::StartTrick { leader: winner });
                    }
                }
                if self.state.phase.is_playing() {
                    broadcast(
                        self,
                        &self.play_status_event(self.state.next_actor.unwrap()),
                    );
                } else {
                    self.finish_hand(broadcast);
                }
            }
            GameEvent::AcceptClaim { claimer, .. } => {
                if self.state.claims.successfully_claimed(*claimer) {
                    self.finish_hand(broadcast);
                }
            }
            _ => {}
        }
    }

    fn finish_hand<F>(&mut self, mut broadcast: F)
    where
        F: FnMut(&mut Game<S>, &GameEvent),
    {
        let scores = self.state.scores();
        broadcast(
            self,
            &GameEvent::HandComplete {
                north_score: scores.score(Seat::North),
                east_score: scores.score(Seat::East),
                south_score: scores.score(Seat::South),
                west_score: scores.score(Seat::West),
            },
        );
        if self.state.phase.is_complete() {
            let seed = if let GameEvent::Sit { seed, .. } = &self.events[0] {
                seed.clone()
            } else {
                panic!("First event must be a sit event");
            };
            broadcast(self, &GameEvent::GameComplete { seed });
        }
    }

    pub fn verify_pass(&self, game_id: GameId, seat: Seat, cards: Cards) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_passing() {
            return Err(RulesError::IllegalAction("pass", self.state.phase));
        }
        if !self.pre_pass_hand[seat.idx()].contains_all(cards) {
            return Err(RulesError::NotYourCards(
                cards - self.pre_pass_hand[seat.idx()],
            ));
        }
        if cards.len() != 3 {
            return Err(RulesError::IllegalPassSize(cards));
        }
        let passed = self.pre_pass_hand[seat.idx()] - self.post_pass_hand[seat.idx()];
        if !passed.is_empty() {
            return Err(RulesError::AlreadyPassed(passed));
        }
        Ok(())
    }

    pub fn verify_charge(
        &self,
        game_id: GameId,
        seat: Seat,
        cards: Cards,
    ) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_charging() {
            return Err(RulesError::IllegalAction("charge", self.state.phase));
        }
        let hand_cards = self.post_pass_hand[seat.idx()];
        if !hand_cards.contains_all(cards) {
            return Err(RulesError::NotYourCards(cards - hand_cards));
        }
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(RulesError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        if self.state.charges.all_charges().contains_any(cards) {
            return Err(RulesError::AlreadyCharged(
                self.state.charges.all_charges() & cards,
            ));
        }
        if !self.state.can_charge(seat) {
            return Err(RulesError::NotYourTurn(
                self.players[self.state.next_actor.unwrap().idx()],
                "charge",
            ));
        }
        Ok(())
    }

    pub fn verify_play(&self, game_id: GameId, seat: Seat, card: Card) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(RulesError::IllegalAction("play", self.state.phase));
        }
        let mut plays = self.post_pass_hand[seat.idx()] - self.state.played;
        if !plays.contains(card) {
            return Err(RulesError::NotYourCards(card.into()));
        }
        if seat != self.state.next_actor.unwrap() {
            return Err(RulesError::NotYourTurn(
                self.players[self.state.next_actor.unwrap().idx()],
                "play",
            ));
        }
        if self.state.led_suits.is_empty() {
            if plays.contains(Card::TwoClubs) && card != Card::TwoClubs {
                return Err(RulesError::MustPlayTwoOfClubs);
            }
            if !Cards::POINTS.contains_all(plays) {
                plays -= Cards::POINTS;
                if !plays.contains(card) {
                    return Err(RulesError::NoPointsOnFirstTrick);
                }
            } else if plays.contains(Card::JackDiamonds) && card != Card::JackDiamonds {
                return Err(RulesError::MustPlayJackOfDiamonds);
            } else if !plays.contains(Card::JackDiamonds)
                && plays.contains(Card::QueenSpades)
                && card != Card::QueenSpades
            {
                return Err(RulesError::MustPlayQueenOfSpades);
            }
        }
        if !self.state.current_trick.is_empty() {
            let suit = self.state.current_trick.suit();
            if suit.cards().contains_any(plays) {
                plays &= suit.cards();
                if !plays.contains(card) {
                    return Err(RulesError::MustFollowSuit);
                }
                if !self.state.led_suits.contains(suit) && plays.len() > 1 {
                    plays -= self.state.charges.all_charges();
                    if !plays.contains(card) {
                        return Err(RulesError::NoChargeOnFirstTrickOfSuit);
                    }
                }
            }
        } else {
            if !self.state.played.contains_any(Cards::HEARTS) && !Cards::HEARTS.contains_all(plays)
            {
                plays -= Cards::HEARTS;
                if !plays.contains(card) {
                    return Err(RulesError::HeartsNotBroken);
                }
            }
            let unled_charges = self.state.charges.all_charges() - self.state.led_suits.cards();
            if !unled_charges.contains_all(plays) {
                plays -= unled_charges;
                if !plays.contains(card) {
                    return Err(RulesError::NoChargeOnFirstTrickOfSuit);
                }
            }
        }
        Ok(())
    }

    pub fn verify_claim(&self, game_id: GameId, seat: Seat) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(RulesError::IllegalAction("claim", self.state.phase));
        }
        if self.state.claims.is_claiming(seat) {
            return Err(RulesError::AlreadyClaiming(self.players[seat.idx()]));
        }
        Ok(())
    }

    pub fn verify_accept_claim(
        &self,
        game_id: GameId,
        claimer: Seat,
        acceptor: Seat,
    ) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(RulesError::IllegalAction("accept claim", self.state.phase));
        }
        if !self.state.claims.is_claiming(claimer) {
            return Err(RulesError::NotClaiming(self.players[claimer.idx()]));
        }
        if self.state.claims.has_accepted(claimer, acceptor) {
            return Err(RulesError::AlreadyAcceptedClaim(
                self.players[acceptor.idx()],
                self.players[claimer.idx()],
            ));
        }
        Ok(())
    }

    pub fn verify_reject_claim(&self, game_id: GameId, claimer: Seat) -> Result<(), RulesError> {
        if self.state.phase.is_complete() {
            return Err(RulesError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(RulesError::IllegalAction("reject claim", self.state.phase));
        }
        if !self.state.claims.is_claiming(claimer) {
            return Err(RulesError::NotClaiming(self.players[claimer.idx()]));
        }
        Ok(())
    }

    pub fn deal_event(&self) -> Option<GameEvent> {
        match self.state.phase {
            GamePhase::PlayLeft => Some(self.seed.deal(PassDirection::Right)),
            GamePhase::PlayRight => Some(self.seed.deal(PassDirection::Across)),
            GamePhase::PlayAcross => Some(self.seed.deal(PassDirection::Keeper)),
            _ => None,
        }
    }
}
