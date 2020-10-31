use crate::{util, BotRunner, Database, Sender};
use log::info;
use rand_distr::Gamma;
use rusqlite::{ToSql, Transaction};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};
use tokio::{
    sync::{mpsc, mpsc::UnboundedReceiver, Mutex},
    task,
};
use turbo_hearts_api::{
    Card, Cards, CardsError, GameEvent, GameId, GamePhase, GameState, HashedSeed, PassDirection,
    Player, PlayerWithOptions, Seat, Seed, UserId,
};

#[derive(Clone)]
pub struct Games {
    db: &'static Database,
    bot_delay: Option<Gamma<f32>>,
    inner: Arc<Mutex<HashMap<GameId, Arc<Mutex<Game>>>>>,
}

impl Games {
    pub fn new(db: &'static Database, delay: bool) -> Self {
        Self {
            db,
            bot_delay: if delay {
                Some(Gamma::new(2.5, 0.8).unwrap())
            } else {
                None
            },
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        let mut unwatched = Vec::new();
        for (game_id, game) in inner.iter() {
            let mut game = game.lock().await;
            game.broadcast(&GameEvent::Ping);
            if game.subscribers.is_empty() {
                unwatched.push(*game_id);
            }
        }
        for game_id in unwatched {
            inner.remove(&game_id);
        }
    }

    async fn with_game<F, T>(&self, game_id: GameId, f: F) -> Result<T, CardsError>
    where
        F: FnOnce(&mut Game) -> Result<T, CardsError>,
    {
        let mut launch_bots = false;
        let game = {
            let mut inner = self.inner.lock().await;
            match inner.entry(game_id) {
                Entry::Occupied(entry) => Arc::clone(entry.get()),
                Entry::Vacant(entry) => {
                    let game = Arc::new(Mutex::new(Game::new()));
                    entry.insert(Arc::clone(&game));
                    launch_bots = true;
                    game
                }
            }
        };
        let mut game = game.lock().await;
        if game.events.is_empty() {
            self.db
                .run_read_only(|tx| hydrate_events(&tx, game_id, &mut game))?;
        }
        if game.events.is_empty() {
            Err(CardsError::UnknownGame(game_id))
        } else {
            if launch_bots {
                if let GameEvent::Sit {
                    north,
                    east,
                    south,
                    west,
                    ..
                } = game.events[0].clone()
                {
                    self.run_bot(game_id, Seat::North, north, &mut game);
                    self.run_bot(game_id, Seat::East, east, &mut game);
                    self.run_bot(game_id, Seat::South, south, &mut game);
                    self.run_bot(game_id, Seat::West, west, &mut game);
                }
            }
            f(&mut game)
        }
    }

    fn run_bot(&self, game_id: GameId, seat: Seat, player: Player, game: &mut Game) {
        if let Player::Bot { user_id, strategy } = player {
            let (tx, rx) = mpsc::unbounded_channel();
            let tx = Sender::new(tx, None);
            task::spawn(BotRunner::new(game_id, user_id, strategy).run(
                self.clone(),
                rx,
                self.bot_delay,
            ));
            self.replay_events(&game, &tx, Some(seat));
            game.bots.push((seat, tx));
            info!(
                "run_bot: game_id={}, seat={}, player={:?}",
                game_id, seat, player
            );
        }
    }

    pub fn start_game(
        &self,
        game_id: GameId,
        players: [PlayerWithOptions; 4],
        seed: Seed,
    ) -> Result<(), CardsError> {
        let hashed_seed = HashedSeed::from(&seed);
        let result = self.db.run_with_retry(|tx| {
            persist_events(
                &tx,
                game_id,
                0,
                &[
                    GameEvent::Sit {
                        north: players[0].player,
                        east: players[1].player,
                        south: players[2].player,
                        west: players[3].player,
                        rules: players[0].rules,
                        seed: seed.clone(),
                    },
                    hashed_seed.deal(PassDirection::Left),
                ],
            )
        });
        info!(
            "start_game: game_id={}, error={:?}",
            game_id,
            result.as_ref().err()
        );
        result
    }

    pub async fn subscribe(
        &self,
        game_id: GameId,
        user_id: UserId,
        last_event_id: Option<usize>,
    ) -> Result<UnboundedReceiver<(GameEvent, usize)>, CardsError> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx = Sender::new(tx, last_event_id);
        self.with_game(game_id, |game| {
            let seat = game.seat(user_id);
            let mut subscribers = game
                .subscribers
                .iter()
                .map(|(user_id, _)| *user_id)
                .collect::<HashSet<_>>();
            if subscribers.insert(user_id) {
                game.broadcast(&GameEvent::JoinGame { user_id });
            }
            self.replay_events(&game, &tx, seat);
            tx.send(GameEvent::EndReplay { subscribers });
            game.subscribers.push((user_id, tx));
            Ok(())
        })
        .await?;
        info!("subscribe: game_id={}, user_id={}", game_id, user_id);
        Ok(rx)
    }

    fn replay_events(&self, game: &Game, tx: &Sender, seat: Option<Seat>) {
        let mut copy = Game::new();
        for event in &game.events {
            copy.apply(event, |g, e| {
                tx.send(e.redact(seat, g.state.rules));
            });
        }
    }

    pub async fn pass_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_pass(game_id, seat, cards)?;
                    let mut events = vec![GameEvent::SendPass { from: seat, cards }];
                    if game.state.phase != GamePhase::PassKeeper {
                        let sender = game.state.phase.pass_sender(seat);
                        if game.state.done.sent_pass(sender) {
                            events.push(GameEvent::RecvPass {
                                to: seat,
                                cards: game.pre_pass_hand[sender.idx()]
                                    - game.post_pass_hand[sender.idx()],
                            });
                        }
                        let receiver = game.state.phase.pass_receiver(seat);
                        if game.state.done.sent_pass(receiver) {
                            events.push(GameEvent::RecvPass {
                                to: receiver,
                                cards,
                            });
                        }
                    }
                    if game.state.phase == GamePhase::PassKeeper
                        && Seat::all(|s| s == seat || game.state.done.sent_pass(s))
                    {
                        let passes = Cards::ALL
                            - game.post_pass_hand[0]
                            - game.post_pass_hand[1]
                            - game.post_pass_hand[2]
                            - game.post_pass_hand[3]
                            | cards;
                        events.extend_from_slice(&game.seed.keeper_pass(passes));
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| g.broadcast(e));
                    }
                    Ok(())
                }
            })
            .await;
        info!(
            "pass: game_id={}, user_id={}, cards={}, error={:?}",
            game_id,
            user_id,
            cards,
            result.as_ref().err()
        );
        result
    }

    pub async fn charge_cards(
        &self,
        game_id: GameId,
        user_id: UserId,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_charge(game_id, seat, cards)?;
                    let event = GameEvent::Charge { seat, cards };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| g.broadcast(e));
                    Ok(())
                }
            })
            .await;
        info!(
            "charge: game_id={}, user_id={}, cards={}, error={:?}",
            game_id,
            user_id,
            cards,
            result.as_ref().err()
        );
        result
    }

    pub async fn play_card(
        &self,
        game_id: GameId,
        user_id: UserId,
        card: Card,
    ) -> Result<bool, CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_play(game_id, seat, card)?;
                    let mut events = vec![GameEvent::Play { seat, card }];
                    let ends_hand = game.state.played | card == Cards::ALL;
                    if ends_hand {
                        game.add_deal_event(&mut events);
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)?;
                        if ends_hand {
                            game.finish_if_keeper(&tx, game_id)?;
                        }
                        Ok(())
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| g.broadcast(e));
                    }
                    Ok(game.state.phase == GamePhase::Complete)
                }
            })
            .await;
        info!(
            "play: game_id={}, user_id={}, card={}, error={:?}",
            game_id,
            user_id,
            card,
            result.as_ref().err()
        );
        result
    }

    pub async fn claim(&self, game_id: GameId, user_id: UserId) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_claim(game_id, seat)?;
                    let event = GameEvent::Claim {
                        seat,
                        hand: game.post_pass_hand[seat.idx()] - game.state.played,
                    };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| g.broadcast(e));
                    Ok(())
                }
            })
            .await;
        info!(
            "claim: game_id={}, user_id={}, error={:?}",
            game_id,
            user_id,
            result.as_ref().err()
        );
        result
    }

    pub async fn accept_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<bool, CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_accept_claim(game_id, claimer, seat)?;
                    let mut events = vec![GameEvent::AcceptClaim {
                        claimer,
                        acceptor: seat,
                    }];
                    let ends_hand = game.state.claims.will_successfully_claim(claimer, seat);
                    if ends_hand {
                        game.add_deal_event(&mut events);
                    }
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &events)?;
                        if ends_hand {
                            game.finish_if_keeper(&tx, game_id)?;
                        }
                        Ok(())
                    })?;
                    for event in events {
                        game.apply(&event, |g, e| g.broadcast(e));
                    }
                    Ok(game.state.phase == GamePhase::Complete)
                }
            })
            .await;
        info!(
            "accept_claim: game_id={}, user_id={}, claimer={}, error={:?}",
            game_id,
            user_id,
            claimer,
            result.as_ref().err()
        );
        result
    }

    pub async fn reject_claim(
        &self,
        game_id: GameId,
        user_id: UserId,
        claimer: Seat,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| match game.seat(user_id) {
                None => Err(CardsError::InvalidPlayer(user_id, game_id)),
                Some(seat) => {
                    game.verify_reject_claim(game_id, claimer)?;
                    let event = GameEvent::RejectClaim {
                        claimer,
                        rejector: seat,
                    };
                    self.db.run_with_retry(|tx| {
                        persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                    })?;
                    game.apply(&event, |g, e| g.broadcast(e));
                    Ok(())
                }
            })
            .await;
        info!(
            "reject_claim: game_id={}, user_id={}, claimer={}, error={:?}",
            game_id,
            user_id,
            claimer,
            result.as_ref().err()
        );
        result
    }

    pub async fn chat(
        &self,
        game_id: GameId,
        user_id: UserId,
        message: String,
    ) -> Result<(), CardsError> {
        let result = self
            .with_game(game_id, |game| {
                let event = GameEvent::Chat { user_id, message };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, game_id, game.events.len(), &[event.clone()])
                })?;
                game.apply(&event, |g, e| g.broadcast(e));
                Ok(())
            })
            .await;
        info!(
            "chat: game_id={}, user_id={}, error={:?}",
            game_id,
            user_id,
            result.as_ref().err()
        );
        result
    }
}

#[derive(Debug)]
pub struct Game {
    pub events: Vec<GameEvent>,
    pub subscribers: Vec<(UserId, Sender)>,
    pub bots: Vec<(Seat, Sender)>,
    pub pre_pass_hand: [Cards; 4],
    pub post_pass_hand: [Cards; 4],
    pub players: [UserId; 4],
    pub state: GameState,
    pub seed: HashedSeed,
}

impl Game {
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

    fn broadcast(&mut self, event: &GameEvent) {
        let rules = self.state.rules;
        let players = self.players;
        let mut disconnects = HashSet::new();
        self.subscribers.retain(|(user_id, tx)| {
            let seat = seat(players, *user_id);
            if tx.send(event.redact(seat, rules)) {
                true
            } else {
                disconnects.insert(*user_id);
                false
            }
        });
        if !disconnects.is_empty() {
            for (user_id, _) in &self.subscribers {
                disconnects.remove(user_id);
            }
            for user_id in disconnects {
                self.broadcast(&GameEvent::LeaveGame { user_id });
            }
        }
        for (seat, bot) in &self.bots {
            bot.send(event.redact(Some(*seat), rules));
        }
    }

    fn seat(&self, user_id: UserId) -> Option<Seat> {
        seat(self.players, user_id)
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
        F: FnMut(&mut Game, &GameEvent),
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
                self.post_pass_hand[from.idx()] -= *cards;
                broadcast(self, &self.state.pass_status_event());
            }
            GameEvent::RecvPass { to, cards } => {
                self.post_pass_hand[to.idx()] |= *cards;
                if self.state.phase.is_charging() {
                    broadcast(self, &GameEvent::StartCharging);
                    broadcast(self, &self.state.charge_status_event());
                }
            }
            GameEvent::Charge { .. } => {
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
            GameEvent::Play { .. } => {
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
        F: FnMut(&mut Game, &GameEvent),
    {
        broadcast(
            self,
            &GameEvent::HandComplete {
                north_score: self.state.score(Seat::North),
                east_score: self.state.score(Seat::East),
                south_score: self.state.score(Seat::South),
                west_score: self.state.score(Seat::West),
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

    fn verify_pass(&self, game_id: GameId, seat: Seat, cards: Cards) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_passing() {
            return Err(CardsError::IllegalAction("pass", self.state.phase));
        }
        if !self.pre_pass_hand[seat.idx()].contains_all(cards) {
            return Err(CardsError::NotYourCards(
                cards - self.pre_pass_hand[seat.idx()],
            ));
        }
        if cards.len() != 3 {
            return Err(CardsError::IllegalPassSize(cards));
        }
        let passed = self.pre_pass_hand[seat.idx()] - self.post_pass_hand[seat.idx()];
        if !passed.is_empty() {
            return Err(CardsError::AlreadyPassed(passed));
        }
        Ok(())
    }

    fn verify_charge(
        &mut self,
        game_id: GameId,
        seat: Seat,
        cards: Cards,
    ) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_charging() {
            return Err(CardsError::IllegalAction("charge", self.state.phase));
        }
        let hand_cards = self.post_pass_hand[seat.idx()];
        if !hand_cards.contains_all(cards) {
            return Err(CardsError::NotYourCards(cards - hand_cards));
        }
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(CardsError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        if self.state.charges.all_charges().contains_any(cards) {
            return Err(CardsError::AlreadyCharged(
                self.state.charges.all_charges() & cards,
            ));
        }
        if !self.state.can_charge(seat) {
            return Err(CardsError::NotYourTurn(
                self.players[self.state.next_actor.unwrap().idx()],
                "charge",
            ));
        }
        Ok(())
    }

    fn verify_play(&mut self, game_id: GameId, seat: Seat, card: Card) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(CardsError::IllegalAction("play", self.state.phase));
        }
        let mut plays = self.post_pass_hand[seat.idx()] - self.state.played;
        if !plays.contains(card) {
            return Err(CardsError::NotYourCards(card.into()));
        }
        if seat != self.state.next_actor.unwrap() {
            return Err(CardsError::NotYourTurn(
                self.players[self.state.next_actor.unwrap().idx()],
                "play",
            ));
        }
        if self.state.led_suits.is_empty() {
            if plays.contains(Card::TwoClubs) && card != Card::TwoClubs {
                return Err(CardsError::MustPlayTwoOfClubs);
            }
            if !Cards::POINTS.contains_all(plays) {
                plays -= Cards::POINTS;
                if !plays.contains(card) {
                    return Err(CardsError::NoPointsOnFirstTrick);
                }
            } else if plays.contains(Card::JackDiamonds) && card != Card::JackDiamonds {
                return Err(CardsError::MustPlayJackOfDiamonds);
            } else if !plays.contains(Card::JackDiamonds)
                && plays.contains(Card::QueenSpades)
                && card != Card::QueenSpades
            {
                return Err(CardsError::MustPlayQueenOfSpades);
            }
        }
        if !self.state.current_trick.is_empty() {
            let suit = self.state.current_trick.suit();
            if suit.cards().contains_any(plays) {
                plays &= suit.cards();
                if !plays.contains(card) {
                    return Err(CardsError::MustFollowSuit);
                }
                if !self.state.led_suits.contains(suit) && plays.len() > 1 {
                    plays -= self.state.charges.all_charges();
                    if !plays.contains(card) {
                        return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                    }
                }
            }
        } else {
            if !self.state.played.contains_any(Cards::HEARTS) && !Cards::HEARTS.contains_all(plays)
            {
                plays -= Cards::HEARTS;
                if !plays.contains(card) {
                    return Err(CardsError::HeartsNotBroken);
                }
            }
            let unled_charges = self.state.charges.all_charges() - self.state.led_suits.cards();
            if !unled_charges.contains_all(plays) {
                plays -= unled_charges;
                if !plays.contains(card) {
                    return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                }
            }
        }
        Ok(())
    }

    fn verify_claim(&mut self, game_id: GameId, seat: Seat) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(CardsError::IllegalAction("claim", self.state.phase));
        }
        if self.state.claims.is_claiming(seat) {
            return Err(CardsError::AlreadyClaiming(self.players[seat.idx()]));
        }
        Ok(())
    }

    fn verify_accept_claim(
        &mut self,
        game_id: GameId,
        claimer: Seat,
        acceptor: Seat,
    ) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(CardsError::IllegalAction("accept claim", self.state.phase));
        }
        if !self.state.claims.is_claiming(claimer) {
            return Err(CardsError::NotClaiming(self.players[claimer.idx()]));
        }
        if self.state.claims.has_accepted(claimer, acceptor) {
            return Err(CardsError::AlreadyAcceptedClaim(
                self.players[acceptor.idx()],
                self.players[claimer.idx()],
            ));
        }
        Ok(())
    }

    fn verify_reject_claim(&mut self, game_id: GameId, claimer: Seat) -> Result<(), CardsError> {
        if self.state.phase.is_complete() {
            return Err(CardsError::GameComplete(game_id));
        }
        if !self.state.phase.is_playing() {
            return Err(CardsError::IllegalAction("reject claim", self.state.phase));
        }
        if !self.state.claims.is_claiming(claimer) {
            return Err(CardsError::NotClaiming(self.players[claimer.idx()]));
        }
        Ok(())
    }

    fn add_deal_event(&self, events: &mut Vec<GameEvent>) {
        match self.state.phase {
            GamePhase::PlayLeft => events.push(self.seed.deal(PassDirection::Right)),
            GamePhase::PlayRight => events.push(self.seed.deal(PassDirection::Across)),
            GamePhase::PlayAcross => events.push(self.seed.deal(PassDirection::Keeper)),
            _ => {}
        };
    }

    fn finish_if_keeper(&self, tx: &Transaction, game_id: GameId) -> Result<(), CardsError> {
        if self.state.phase == GamePhase::PlayKeeper {
            tx.execute::<&[&dyn ToSql]>(
                "UPDATE game SET completed_time = ? WHERE game_id = ?",
                &[&util::timestamp(), &game_id],
            )?;
        }
        Ok(())
    }
}

fn seat(players: [UserId; 4], user_id: UserId) -> Option<Seat> {
    players
        .iter()
        .position(|&id| id == user_id)
        .map(|idx| Seat::VALUES[idx])
}

pub fn persist_events(
    tx: &Transaction,
    game_id: GameId,
    event_id: usize,
    events: &[GameEvent],
) -> Result<(), CardsError> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO event (game_id, event_id, timestamp, event) VALUES (?, ?, ?, ?)",
    )?;
    let timestamp = util::timestamp();
    let mut event_id = event_id as isize;
    for event in events {
        stmt.execute::<&[&dyn ToSql]>(&[&game_id, &event_id, &timestamp, &event])?;
        event_id += 1;
    }
    Ok(())
}

fn hydrate_events(tx: &Transaction, game_id: GameId, game: &mut Game) -> Result<(), CardsError> {
    let mut stmt = tx.prepare_cached(
        "SELECT event FROM event WHERE game_id = ? AND event_id >= ? ORDER BY event_id",
    )?;
    let mut rows = stmt.query::<&[&dyn ToSql]>(&[&game_id, &(game.events.len() as i64)])?;
    while let Some(row) = rows.next()? {
        let event = serde_json::from_str(&row.get::<_, String>(0)?)?;
        game.apply(&event, |_, _| {});
    }
    Ok(())
}
