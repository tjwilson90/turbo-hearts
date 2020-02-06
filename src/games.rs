use crate::{
    cards::{Card, Cards, Suit},
    db::Database,
    error::CardsError,
    hacks::{Mutex, UnboundedSender},
    types::{ChargingRules, EventId, GameId, Hand, Player, Seat},
};
use rand::seq::SliceRandom;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    OptionalExtension, ToSql, Transaction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
    time::SystemTime,
};

#[derive(Clone)]
pub struct Games {
    db: Database,
    inner: Arc<Mutex<HashMap<GameId, Arc<Mutex<Option<GameState>>>>>>,
}

impl Games {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        let mut unwatched = Vec::new();
        for (id, game) in inner.iter_mut() {
            let mut game = game.lock().await;
            match game.as_mut() {
                Some(game) => {
                    game.broadcast_public(&Event {
                        id: 0,
                        seat: Seat::North,
                        timestamp: 0,
                        kind: EventKind::Ping,
                    });
                    if game.subscribers.is_empty() {
                        unwatched.push(*id);
                    }
                }
                None => unwatched.push(*id),
            }
        }
        for id in unwatched {
            inner.remove(&id);
        }
    }

    async fn with_game<F>(&self, id: GameId, f: F) -> Result<(), CardsError>
    where
        F: FnOnce(&mut GameState) -> Result<(), CardsError>,
    {
        let game = {
            let mut inner = self.inner.lock().await;
            match inner.entry(id) {
                Entry::Occupied(entry) => entry.get().clone(),
                Entry::Vacant(entry) => {
                    let game = Arc::new(Mutex::new(None));
                    entry.insert(game.clone());
                    game
                }
            }
        };
        let mut game = game.lock().await;
        if game.is_none() {
            *game = Some(self.db.run_read_only(|tx| {
                let mut game = self.load_game(&tx, id)?;
                self.hydrate_events(&tx, id, &mut game)?;
                Ok(game)
            })?)
        } else {
            self.db
                .run_read_only(|tx| self.hydrate_events(&tx, id, game.as_mut().unwrap()))?;
        }
        f(game.as_mut().unwrap())
    }

    fn load_game(&self, tx: &Transaction, id: GameId) -> Result<GameState, CardsError> {
        tx.query_row(
            "SELECT north, east, south, west, rules FROM game WHERE id = ?",
            &[&id],
            |row| {
                Ok(GameState::new(
                    id,
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?
        .ok_or(CardsError::UnknownGame(id))
    }

    fn persist_events(
        &self,
        tx: &Transaction,
        id: GameId,
        events: &[Event],
    ) -> Result<(), CardsError> {
        let mut stmt = tx.prepare(
            "INSERT INTO event (game_id, event_id, hand, seat, timestamp, kind)
                VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        for event in events {
            stmt.execute::<&[&dyn ToSql]>(&[
                &id,
                &event.id,
                &event.seat,
                &event.timestamp,
                &event.kind,
            ])?;
        }
        Ok(())
    }

    fn hydrate_events(
        &self,
        tx: &Transaction,
        id: GameId,
        game: &mut GameState,
    ) -> Result<(), CardsError> {
        let mut stmt = tx.prepare(
            "SELECT event_id, seat, timestamp, kind FROM event
            WHERE game_id = ? AND event_id >= ? ORDER BY event_id",
        )?;
        let mut rows = stmt.query::<&[&dyn ToSql]>(&[&id, &(game.events.len() as i64)])?;
        while let Some(row) = rows.next()? {
            game.apply(Event {
                id: row.get(0)?,
                seat: row.get(1)?,
                timestamp: row.get(2)?,
                kind: serde_json::from_str(&row.get::<_, String>(3)?)?,
            });
        }
        Ok(())
    }

    pub fn start_game(
        &self,
        id: GameId,
        players: &HashMap<Player, ChargingRules>,
    ) -> Result<(), CardsError> {
        let mut order = players.keys().cloned().collect::<Vec<_>>();
        order.shuffle(&mut rand::thread_rng());
        let timestamp = timestamp();
        let deal = deal(0, timestamp);
        self.db.run_with_retry(|tx| {
            tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO game (id, timestamp, north, east, south, west, rules)
                VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    &id,
                    &timestamp,
                    &order[0],
                    &order[1],
                    &order[2],
                    &order[3],
                    &players.get(&order[0]).unwrap(),
                ],
            )?;
            self.persist_events(&tx, id, &deal)
        })?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        id: GameId,
        player: Player,
        tx: UnboundedSender<Event>,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| {
            for event in &game.events {
                tx.send(event.clone()).unwrap();
            }
            game.subscribers.insert(player, tx);
            Ok(())
        })
        .await
    }

    pub async fn pass_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match seat(&game.players, &player) {
            Some(seat) => {
                let event = game.pass_cards(seat, cards)?;
                self.db
                    .run_with_retry(|tx| self.persist_events(&tx, id, &[event.clone()]))?;
                game.broadcast(&event);
                game.apply(event);
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(player)),
        })
        .await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match seat(&game.players, &player) {
            Some(seat) => {
                let mut events = vec![game.charge_cards(seat, cards)?];
                if game.rules.blind() && cards.is_empty() {
                    let hand = &game.hands[game.hand().unwrap().idx()];
                    if hand.done_charging[seat.next().idx()]
                        && hand.done_charging[seat.next().next().idx()]
                        && hand.done_charging[seat.next().next().next().idx()]
                    {
                        let mut id = events[0].id;
                        let timestamp = events[0].timestamp;
                        for seat in &Seat::VALUES {
                            if !(hand.charges & hand.post_pass_hand[seat.idx()]).is_empty() {
                                id += 1;
                                events.push(Event {
                                    id,
                                    seat: *seat,
                                    timestamp,
                                    kind: EventKind::ChargeCards {
                                        charges: hand.charges & hand.post_pass_hand[seat.idx()],
                                    },
                                })
                            }
                        }
                    }
                }
                self.db
                    .run_with_retry(|tx| self.persist_events(&tx, id, &events))?;
                let mut public = false;
                for event in events {
                    if public {
                        game.broadcast_public(&event);
                    } else {
                        game.broadcast(&event);
                        public = true;
                    }
                    game.apply(event);
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(player)),
        })
        .await
    }

    pub async fn play_card(
        &self,
        id: GameId,
        player: Player,
        card: Card,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match seat(&game.players, &player) {
            Some(seat) => {
                let event = game.play_card(seat, card)?;
                self.db
                    .run_with_retry(|tx| self.persist_events(&tx, id, &[event.clone()]))?;
                game.broadcast(&event);
                game.apply(event);
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(player)),
        })
        .await
    }
}

struct GameState {
    id: GameId,
    rules: ChargingRules,
    subscribers: HashMap<Player, UnboundedSender<Event>>,
    players: [Player; 4],
    hands: [HandState; 4],
    events: Vec<Event>,
}

impl GameState {
    fn new(
        id: GameId,
        north: Player,
        east: Player,
        south: Player,
        west: Player,
        rules: ChargingRules,
    ) -> Self {
        Self {
            id,
            rules,
            subscribers: HashMap::new(),
            players: [north, east, south, west],
            hands: [
                HandState::new(Hand::Left, rules),
                HandState::new(Hand::Right, rules),
                HandState::new(Hand::Across, rules),
                HandState::new(Hand::Keeper, rules),
            ],
            events: Vec::new(),
        }
    }

    fn broadcast(&mut self, event: &Event) {
        match event.kind {
            EventKind::Ping | EventKind::PlayCard { .. } => self.broadcast_public(event),
            EventKind::ReceiveHand { .. } | EventKind::ReceivePass { .. } => {
                self.broadcast_private(event, None);
            }
            EventKind::BlindCharge { .. } => unreachable!(),
            EventKind::ChargeCards { charges } => {
                if self.rules.blind() {
                    self.broadcast_private(
                        event,
                        Some(&Event {
                            id: event.id,
                            seat: event.seat,
                            timestamp: event.timestamp,
                            kind: EventKind::BlindCharge {
                                count: charges.len(),
                            },
                        }),
                    );
                } else {
                    self.broadcast_public(event);
                }
            }
        }
    }

    fn broadcast_private(&mut self, private: &Event, public: Option<&Event>) {
        let players = &self.players;
        self.subscribers.retain(|p, tx| {
            let seat = seat(&players, p);
            if seat.is_none() || seat == Some(private.seat) {
                tx.send(private.clone()).is_ok()
            } else if let Some(event) = public {
                tx.send(event.clone()).is_ok()
            } else {
                true
            }
        })
    }

    fn broadcast_public(&mut self, event: &Event) {
        self.subscribers
            .retain(|_, tx| tx.send(event.clone()).is_ok())
    }

    fn hand(&self) -> Option<Hand> {
        for hand in &Hand::VALUES {
            if self.hands[hand.idx()].status != HandStatus::Complete {
                return Some(*hand);
            }
        }
        None
    }

    fn apply(&mut self, event: Event) {
        let state: &mut HandState = &mut self.hands[self.hand().unwrap().idx()];
        let seat = event.seat.idx();
        match event.kind {
            EventKind::Ping => unreachable!(),
            EventKind::ReceiveHand { hand } => {
                state.pre_pass_hand[seat] = hand;
                state.post_pass_hand[seat] = hand;
            }
            EventKind::ReceivePass { pass } => {
                state.received_pass[seat] = pass;
                for post_pass_hand in &mut state.post_pass_hand {
                    *post_pass_hand -= pass;
                }
                state.post_pass_hand[seat] |= pass;
                if state.received_pass.iter().all(|pass| pass.len() > 0) {
                    state.status = HandStatus::Charging;
                }
            }
            EventKind::BlindCharge { .. } => unreachable!(),
            EventKind::ChargeCards { charges } => {
                state.charges |= charges;
                if let Some(charger) = &mut state.next_charger {
                    *charger = charger.next();
                }
                if charges.is_empty() {
                    state.done_charging[seat] = true;
                } else {
                    for done_charging in &mut state.done_charging {
                        *done_charging = false;
                    }
                    state.done_charging[seat] = !self.rules.chain();
                }
                if state.done_charging.iter().all(|done| *done) {
                    match state.status {
                        HandStatus::KeeperCharging => {
                            if state.charges.is_empty() {
                                state.status = HandStatus::Passing;
                                if let Some(charger) = &mut state.next_charger {
                                    *charger = Seat::North;
                                }
                            } else {
                                state.start_playing();
                            }
                        }
                        HandStatus::Charging => {
                            state.start_playing();
                        }
                        _ => unreachable!(),
                    }
                }
            }
            EventKind::PlayCard { card } => {
                state.played |= card;
                if state.trick.cards.is_empty() {
                    state.leads |= card;
                }
                state.trick.play(card);
                if let Some(winning_card) = state.trick.winner() {
                    let winner = state.owner(winning_card);
                    state.won[winner.idx()] |= state.trick.cards;
                    state.trick = Trick::new(winner);
                }
                if state.played == Cards::ALL {
                    state.status = HandStatus::Complete;
                }
            }
        }
        self.events.push(event);
    }

    fn pass_cards(&self, seat: Seat, cards: Cards) -> Result<Event, CardsError> {
        let hand = match self.hand() {
            Some(hand) => hand,
            None => return Err(CardsError::GameComplete(self.id)),
        };
        let state = &self.hands[hand.idx()];
        if state.status != HandStatus::Passing {
            return Err(CardsError::IllegalAction(state.status));
        }
        if cards.len() != 3 {
            return Err(CardsError::IllegalPassSize(cards));
        }
        if !state.pre_pass_hand[seat.idx()].contains_all(cards) {
            return Err(CardsError::NotYourCards(
                cards - state.pre_pass_hand[seat.idx()],
            ));
        }
        let receiver = seat.pass_receiver(hand);
        let previous_pass = state.received_pass[receiver.idx()];
        if !previous_pass.is_empty() {
            return Err(CardsError::AlreadyPassed(previous_pass));
        }
        Ok(Event {
            id: self.events.len() as EventId,
            seat: receiver,
            timestamp: timestamp(),
            kind: EventKind::ReceivePass { pass: cards },
        })
    }

    fn charge_cards(&mut self, seat: Seat, cards: Cards) -> Result<Event, CardsError> {
        let hand = match self.hand() {
            Some(hand) => hand,
            None => return Err(CardsError::GameComplete(self.id)),
        };
        let state = &self.hands[hand.idx()];
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(CardsError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        let hand_cards = match state.status {
            HandStatus::KeeperCharging | HandStatus::Charging => state.post_pass_hand[seat.idx()],
            _ => return Err(CardsError::IllegalAction(state.status)),
        };
        if !hand_cards.contains_all(cards) {
            return Err(CardsError::NotYourCards(cards - hand_cards));
        }
        if state.charges.contains_any(cards) {
            return Err(CardsError::AlreadyCharged(state.charges & cards));
        }
        match state.next_charger {
            Some(s) if s != seat => {
                return Err(CardsError::NotYourTurn(self.players[s.idx()].clone()))
            }
            _ => {}
        }
        Ok(Event {
            id: self.events.len() as EventId,
            seat,
            timestamp: timestamp(),
            kind: EventKind::ChargeCards { charges: cards },
        })
    }

    fn play_card(&mut self, seat: Seat, card: Card) -> Result<Event, CardsError> {
        let hand = match self.hand() {
            Some(hand) => hand,
            None => return Err(CardsError::GameComplete(self.id)),
        };
        let state = &self.hands[hand.idx()];
        if !(state.post_pass_hand[seat.idx()] - state.played).contains(card) {
            return Err(CardsError::NotYourCards(card.into()));
        }
        if seat != state.trick.next_player {
            return Err(CardsError::NotYourTurn(
                self.players[state.trick.next_player.idx()].clone(),
            ));
        }
        if state.played.is_empty() && card != Card::TwoClubs {
            return Err(CardsError::MustStartWithTwoOfClubs);
        }
        let current_hand = state.post_pass_hand[seat.idx()] - state.played;
        if state.trick.cards.is_empty() {
            if card.suit() == Suit::Hearts
                && !state.played.contains_any(Cards::HEARTS)
                && !Cards::HEARTS.contains_all(current_hand)
            {
                return Err(CardsError::HeartsNotBroken);
            }
        } else {
            if state.trick.cards.contains(Card::TwoClubs)
                && Cards::POINTS.contains(card)
                && !Cards::POINTS.contains_all(current_hand)
            {
                return Err(CardsError::NoPointsOnFirstTrick);
            }
            let suit = state.trick.lead.unwrap().cards();
            if !suit.contains(card) && current_hand.contains_any(suit) {
                return Err(CardsError::MustFollowSuit);
            }
        }
        if Cards::CHARGEABLE.contains(card)
            && !state.leads.contains_any(card.suit().cards())
            && (current_hand & card.suit().cards()).len() > 1
        {
            return Err(CardsError::NoChargeOnFirstTrickOfSuit);
        }
        Ok(Event {
            id: self.events.len() as EventId,
            seat,
            timestamp: timestamp(),
            kind: EventKind::PlayCard { card },
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandStatus {
    KeeperCharging,
    Passing,
    Charging,
    Playing,
    Complete,
}

struct HandState {
    status: HandStatus,
    pre_pass_hand: [Cards; 4],

    received_pass: [Cards; 4],
    post_pass_hand: [Cards; 4],

    next_charger: Option<Seat>,
    done_charging: [bool; 4],
    charges: Cards,

    leads: Cards,
    played: Cards,
    won: [Cards; 4],
    trick: Trick,
}

impl HandState {
    fn new(hand: Hand, rules: ChargingRules) -> Self {
        Self {
            status: if hand == Hand::Keeper {
                HandStatus::KeeperCharging
            } else {
                HandStatus::Passing
            },
            pre_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            received_pass: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            next_charger: if rules.free() {
                None
            } else {
                Some(Seat::North)
            },
            done_charging: [false, false, false, false],
            charges: Cards::NONE,
            leads: Cards::NONE,
            played: Cards::NONE,
            won: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            trick: Trick::new(Seat::North),
        }
    }

    fn owner(&self, card: Card) -> Seat {
        for (idx, cards) in self.post_pass_hand.iter().enumerate() {
            if cards.contains(card) {
                return Seat::VALUES[idx];
            }
        }
        unreachable!()
    }

    fn start_playing(&mut self) {
        self.status = HandStatus::Playing;
        self.trick = Trick::new(self.owner(Card::TwoClubs));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub seat: Seat,
    pub timestamp: i64,
    pub kind: EventKind,
}

impl Event {
    pub fn is_ping(&self) -> bool {
        match self.kind {
            EventKind::Ping => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventKind {
    Ping,
    ReceiveHand { hand: Cards },
    ReceivePass { pass: Cards },
    BlindCharge { count: u32 },
    ChargeCards { charges: Cards },
    PlayCard { card: Card },
}

impl ToSql for EventKind {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(self).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl FromSql for EventKind {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| serde_json::from_str(s).unwrap())
    }
}

struct Trick {
    next_player: Seat,
    lead: Option<Suit>,
    cards: Cards,
}

impl Trick {
    fn new(leader: Seat) -> Self {
        Self {
            next_player: leader,
            lead: None,
            cards: Cards::NONE,
        }
    }

    fn play(&mut self, card: Card) {
        self.next_player = self.next_player.next();
        if self.lead.is_none() {
            self.lead = Some(card.suit());
        }
        self.cards |= card;
    }

    fn winner(&self) -> Option<Card> {
        let complete = match self.cards.len() {
            8 => true,
            4 => !self.cards.contains(self.lead.unwrap().nine()),
            _ => false,
        };
        if complete {
            Some((self.cards & self.lead.unwrap().cards()).max())
        } else {
            None
        }
    }
}

fn deal(event_id: EventId, timestamp: i64) -> [Event; 4] {
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    [
        Event {
            id: event_id,
            seat: Seat::North,
            timestamp,
            kind: EventKind::ReceiveHand {
                hand: deck[0..13].iter().cloned().collect(),
            },
        },
        Event {
            id: event_id + 1,
            seat: Seat::East,
            timestamp,
            kind: EventKind::ReceiveHand {
                hand: deck[13..26].iter().cloned().collect(),
            },
        },
        Event {
            id: event_id + 2,
            seat: Seat::South,
            timestamp,
            kind: EventKind::ReceiveHand {
                hand: deck[26..39].iter().cloned().collect(),
            },
        },
        Event {
            id: event_id + 3,
            seat: Seat::West,
            timestamp,
            kind: EventKind::ReceiveHand {
                hand: deck[39..52].iter().cloned().collect(),
            },
        },
    ]
}

fn seat(players: &[Player; 4], player: &Player) -> Option<Seat> {
    players
        .iter()
        .position(|p| p == player)
        .map(|idx| Seat::VALUES[idx])
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
