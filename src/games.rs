use crate::{
    cards::{Card, Cards, Suit},
    db::Database,
    error::CardsError,
    hacks::{Mutex, UnboundedSender},
    types::{ChargingRules, EventId, GameId, PassDirection, Player, Seat},
};
use rand::seq::SliceRandom;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql, Transaction,
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
    inner: Arc<Mutex<HashMap<GameId, Arc<Mutex<Game>>>>>,
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
            game.broadcast(&Event::Ping);
            if game.subscribers.is_empty() || game.state == GameState::Complete {
                unwatched.push(*id);
            }
        }
        for id in unwatched {
            inner.remove(&id);
        }
    }

    async fn with_game<F>(&self, id: GameId, f: F) -> Result<(), CardsError>
    where
        F: FnOnce(&mut Game) -> Result<(), CardsError>,
    {
        let game = {
            let mut inner = self.inner.lock().await;
            match inner.entry(id) {
                Entry::Occupied(entry) => entry.get().clone(),
                Entry::Vacant(entry) => {
                    let game = Arc::new(Mutex::new(Game::new()));
                    entry.insert(game.clone());
                    game
                }
            }
        };
        let mut game = game.lock().await;
        self.db
            .run_read_only(|tx| hydrate_events(&tx, id, &mut game))?;
        if game.events.is_empty() {
            Err(CardsError::UnknownGame(id))
        } else {
            f(&mut game)
        }
    }

    pub fn start_game(
        &self,
        id: GameId,
        players: &HashMap<Player, ChargingRules>,
    ) -> Result<(), CardsError> {
        let mut order = players.keys().collect::<Vec<_>>();
        order.shuffle(&mut rand::thread_rng());
        self.db.run_with_retry(|tx| {
            persist_events(
                &tx,
                id,
                0,
                &[
                    Event::Sit {
                        north: order[0].clone(),
                        east: order[0].clone(),
                        south: order[0].clone(),
                        west: order[0].clone(),
                        rules: *players.get(order[0]).unwrap(),
                    },
                    deal(),
                ],
            )
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
            let seat = game.seat(&player);
            for event in &game.events {
                send_event(game.rules, seat, &tx, event);
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
        self.with_game(id, |game| match game.seat(&player) {
            Some(seat) => {
                let event = game.pass_cards(id, seat, cards)?;
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &[event.clone()])
                })?;
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
        self.with_game(id, |game| match game.seat(&player) {
            Some(seat) => {
                let mut events = vec![game.charge_cards(id, seat, cards)?];
                if game.rules.blind() && cards.is_empty() {
                    if game.done_charging[seat.next().idx()]
                        && game.done_charging[seat.next().next().idx()]
                        && game.done_charging[seat.next().next().next().idx()]
                    {
                        for seat in &Seat::VALUES {
                            let charges = game.charges & game.post_pass_hand[seat.idx()];
                            if !charges.is_empty() {
                                events.push(Event::Charge {
                                    seat: *seat,
                                    cards: charges,
                                })
                            }
                        }
                    }
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &events)
                })?;
                let mut events = events.into_iter();
                if let Some(event) = events.next() {
                    game.broadcast(&event);
                    game.apply(event);
                }
                while let Some(event) = events.next() {
                    game.broadcast_public(&event);
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
        self.with_game(id, |game| match game.seat(&player) {
            Some(seat) => {
                let mut events = vec![game.play_card(id, seat, card)?];
                if game.played | card == Cards::ALL && game.pass_direction != PassDirection::Keeper
                {
                    events.push(deal());
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &events)
                })?;
                for event in events {
                    game.broadcast(&event);
                    game.apply(event);
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(player)),
        })
        .await
    }
}

#[derive(Debug)]
struct Game {
    events: Vec<Event>,
    subscribers: HashMap<Player, UnboundedSender<Event>>,
    players: [Player; 4],
    rules: ChargingRules,

    state: GameState,
    pre_pass_hand: [Cards; 4],

    pass_direction: PassDirection,
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

impl Game {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            subscribers: HashMap::new(),
            players: [String::new(), String::new(), String::new(), String::new()],
            rules: ChargingRules::Classic,

            state: GameState::Passing,
            pre_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],

            pass_direction: PassDirection::Left,
            received_pass: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],

            next_charger: None,
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

    fn broadcast(&mut self, event: &Event) {
        let rules = self.rules;
        let players = self.players.clone();
        self.subscribers.retain(|p, tx| {
            let seat = seat(&players, p);
            send_event(rules, seat, tx, event)
        });
    }

    fn broadcast_public(&mut self, event: &Event) {
        self.subscribers
            .retain(|_, tx| tx.send(event.clone()).is_ok())
    }

    fn seat(&self, player: &Player) -> Option<Seat> {
        seat(&self.players, player)
    }

    fn apply(&mut self, event: Event) {
        match event.clone() {
            Event::Ping => unreachable!(),
            Event::Sit {
                north,
                east,
                south,
                west,
                rules,
            } => {
                self.players = [north, east, south, west];
                self.rules = rules;
                if !rules.free() {
                    self.next_charger = Some(Seat::North);
                }
            }
            Event::Deal {
                north,
                east,
                south,
                west,
            } => {
                self.pre_pass_hand[Seat::North.idx()] = north;
                self.post_pass_hand[Seat::North.idx()] = north;
                self.pre_pass_hand[Seat::East.idx()] = east;
                self.post_pass_hand[Seat::East.idx()] = east;
                self.pre_pass_hand[Seat::South.idx()] = south;
                self.post_pass_hand[Seat::South.idx()] = south;
                self.pre_pass_hand[Seat::West.idx()] = west;
                self.post_pass_hand[Seat::West.idx()] = west;
            }
            Event::Pass { from, to, cards } => {
                self.received_pass[to.idx()] = cards;
                self.post_pass_hand[from.idx()] -= cards;
                self.post_pass_hand[to.idx()] |= cards;
                if self.received_pass.iter().all(|pass| pass.len() > 0) {
                    self.state = GameState::Charging;
                }
            }
            Event::BlindCharge { .. } => unreachable!(),
            Event::Charge { seat, cards } => {
                self.charges |= cards;
                if let Some(charger) = &mut self.next_charger {
                    *charger = charger.next();
                }
                if cards.is_empty() {
                    self.done_charging[seat.idx()] = true;
                } else {
                    for done_charging in &mut self.done_charging {
                        *done_charging = false;
                    }
                    self.done_charging[seat.idx()] = !self.rules.chain();
                }
                if self.done_charging.iter().all(|done| *done) {
                    match self.state {
                        GameState::KeeperCharging => {
                            if self.charges.is_empty() {
                                self.state = GameState::Passing;
                                self.next_charger = self.next_charger.map(|_| Seat::North);
                            } else {
                                self.state = GameState::Playing;
                                self.trick = Trick::new(self.owner(Card::TwoClubs));
                            }
                        }
                        GameState::Charging => {
                            self.state = GameState::Playing;
                            self.trick = Trick::new(self.owner(Card::TwoClubs));
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Event::Play { seat: _, card } => {
                self.played |= card;
                if self.trick.cards.is_empty() {
                    self.leads |= card;
                }
                self.trick.play(card);
                if let Some(winning_card) = self.trick.winner() {
                    let winner = self.owner(winning_card);
                    self.won[winner.idx()] |= self.trick.cards;
                    self.trick = Trick::new(winner);
                }
                if self.played == Cards::ALL {
                    match self.pass_direction.next() {
                        Some(pass_direction) => {
                            self.state = if pass_direction == PassDirection::Keeper {
                                GameState::KeeperCharging
                            } else {
                                GameState::Passing
                            };
                            self.pre_pass_hand =
                                [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE];

                            self.pass_direction = pass_direction;
                            self.received_pass =
                                [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE];
                            self.post_pass_hand =
                                [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE];

                            self.next_charger = self.next_charger.map(|_| Seat::North);
                            self.done_charging = [false, false, false, false];
                            self.charges = Cards::NONE;

                            self.leads = Cards::NONE;
                            self.played = Cards::NONE;
                            self.won = [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE];
                        }
                        None => self.state = GameState::Complete,
                    }
                }
            }
        }
        self.events.push(event);
    }

    fn pass_cards(&self, id: GameId, seat: Seat, cards: Cards) -> Result<Event, CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        if self.state != GameState::Passing {
            return Err(CardsError::IllegalAction(self.state));
        }
        if cards.len() != 3 {
            return Err(CardsError::IllegalPassSize(cards));
        }
        if !self.pre_pass_hand[seat.idx()].contains_all(cards) {
            return Err(CardsError::NotYourCards(
                cards - self.pre_pass_hand[seat.idx()],
            ));
        }
        let receiver = seat.pass_receiver(self.pass_direction);
        let previous_pass = self.received_pass[receiver.idx()];
        if !previous_pass.is_empty() {
            return Err(CardsError::AlreadyPassed(previous_pass));
        }
        Ok(Event::Pass {
            from: seat,
            to: receiver,
            cards,
        })
    }

    fn charge_cards(&mut self, id: GameId, seat: Seat, cards: Cards) -> Result<Event, CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(CardsError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        let hand_cards = match self.state {
            GameState::KeeperCharging | GameState::Charging => self.post_pass_hand[seat.idx()],
            _ => return Err(CardsError::IllegalAction(self.state)),
        };
        if !hand_cards.contains_all(cards) {
            return Err(CardsError::NotYourCards(cards - hand_cards));
        }
        if self.charges.contains_any(cards) {
            return Err(CardsError::AlreadyCharged(self.charges & cards));
        }
        match self.next_charger {
            Some(s) if s != seat => {
                return Err(CardsError::NotYourTurn(self.players[s.idx()].clone()))
            }
            _ => {}
        }
        Ok(Event::Charge { seat, cards })
    }

    fn play_card(&mut self, id: GameId, seat: Seat, card: Card) -> Result<Event, CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        let current_hand = self.post_pass_hand[seat.idx()] - self.played;
        if !current_hand.contains(card) {
            return Err(CardsError::NotYourCards(card.into()));
        }
        if seat != self.trick.next_player {
            return Err(CardsError::NotYourTurn(
                self.players[self.trick.next_player.idx()].clone(),
            ));
        }
        if self.played.is_empty() && card != Card::TwoClubs {
            return Err(CardsError::MustStartWithTwoOfClubs);
        }
        if self.trick.cards.is_empty() {
            if card.suit() == Suit::Hearts
                && !self.played.contains_any(Cards::HEARTS)
                && !Cards::HEARTS.contains_all(current_hand)
            {
                return Err(CardsError::HeartsNotBroken);
            }
        } else {
            if self.trick.cards.contains(Card::TwoClubs)
                && Cards::POINTS.contains(card)
                && !Cards::POINTS.contains_all(current_hand)
            {
                return Err(CardsError::NoPointsOnFirstTrick);
            }
            let suit = self.trick.lead.unwrap().cards();
            if !suit.contains(card) && current_hand.contains_any(suit) {
                return Err(CardsError::MustFollowSuit);
            }
        }
        if Cards::CHARGEABLE.contains(card)
            && !self.leads.contains_any(card.suit().cards())
            && (current_hand & card.suit().cards()).len() > 1
        {
            return Err(CardsError::NoChargeOnFirstTrickOfSuit);
        }
        Ok(Event::Play { seat, card })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameState {
    KeeperCharging,
    Passing,
    Charging,
    Playing,
    Complete,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Ping,
    Sit {
        north: Player,
        east: Player,
        south: Player,
        west: Player,
        rules: ChargingRules,
    },
    Deal {
        north: Cards,
        east: Cards,
        south: Cards,
        west: Cards,
    },
    Pass {
        from: Seat,
        to: Seat,
        cards: Cards,
    },
    BlindCharge {
        seat: Seat,
        count: usize,
    },
    Charge {
        seat: Seat,
        cards: Cards,
    },
    Play {
        seat: Seat,
        card: Card,
    },
}

impl Event {
    pub fn is_ping(&self) -> bool {
        match self {
            Event::Ping => true,
            _ => false,
        }
    }
}

impl ToSql for Event {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(self).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl FromSql for Event {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| serde_json::from_str(s).unwrap())
    }
}

#[derive(Debug)]
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

fn deal() -> Event {
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    Event::Deal {
        north: deck[0..13].iter().cloned().collect(),
        east: deck[13..26].iter().cloned().collect(),
        south: deck[26..39].iter().cloned().collect(),
        west: deck[39..52].iter().cloned().collect(),
    }
}

fn seat(players: &[Player; 4], player: &Player) -> Option<Seat> {
    players
        .iter()
        .position(|p| p == player)
        .map(|idx| Seat::VALUES[idx])
}

fn persist_events(
    tx: &Transaction,
    id: GameId,
    mut event_id: EventId,
    events: &[Event],
) -> Result<(), CardsError> {
    let mut stmt = tx.prepare(
        "INSERT INTO event (game_id, event_id, timestamp, event)
                VALUES (?, ?, ?, ?)",
    )?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    for event in events {
        stmt.execute::<&[&dyn ToSql]>(&[&id, &event_id, &timestamp, &event])?;
        event_id += 1;
    }
    Ok(())
}

fn hydrate_events(tx: &Transaction, id: GameId, game: &mut Game) -> Result<(), CardsError> {
    let mut stmt = tx
        .prepare("SELECT event FROM event WHERE game_id = ? AND event_id >= ? ORDER BY event_id")?;
    let mut rows = stmt.query::<&[&dyn ToSql]>(&[&id, &(game.events.len() as i64)])?;
    while let Some(row) = rows.next()? {
        game.apply(serde_json::from_str(&row.get::<_, String>(0)?)?);
    }
    Ok(())
}

fn send_event(
    rules: ChargingRules,
    seat: Option<Seat>,
    tx: &UnboundedSender<Event>,
    event: &Event,
) -> bool {
    let event = match event {
        Event::Ping | Event::Play { .. } | Event::Sit { .. } => event.clone(),
        Event::Deal {
            north,
            east,
            south,
            west,
        } => match seat {
            None => event.clone(),
            Some(seat) => match seat {
                Seat::North => Event::Deal {
                    north: *north,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: Cards::NONE,
                },
                Seat::East => Event::Deal {
                    north: Cards::NONE,
                    east: *east,
                    south: Cards::NONE,
                    west: Cards::NONE,
                },
                Seat::South => Event::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: *south,
                    west: Cards::NONE,
                },
                Seat::West => Event::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: *west,
                },
            },
        },
        Event::Pass { from, to, cards: _ } => match seat {
            Some(seat) if seat != *from && seat != *to => Event::Pass {
                from: *from,
                to: *to,
                cards: Cards::NONE,
            },
            _ => event.clone(),
        },
        Event::BlindCharge { .. } => unreachable!(),
        Event::Charge {
            seat: charger,
            cards,
        } => {
            if rules.blind() {
                match seat {
                    Some(seat) if seat != *charger => Event::BlindCharge {
                        seat: *charger,
                        count: cards.len(),
                    },
                    _ => event.clone(),
                }
            } else {
                event.clone()
            }
        }
    };
    tx.send(event).is_ok()
}
