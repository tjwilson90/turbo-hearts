use crate::cards::{ChargeState, HandState};
use crate::types::{Name, Participant};
use crate::{
    cards::{Card, Cards, Suit},
    db::Database,
    error::CardsError,
    hacks::{unbounded_channel, Mutex, UnboundedReceiver, UnboundedSender},
    types::{ChargingRules, Event, EventId, GameId, PassDirection, Player, Seat},
};
use rand::seq::SliceRandom;
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql, Transaction,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
            game.broadcast(&GameFeEvent::Ping);
            if game.subscribers.is_empty() || game.state == GameState::Complete {
                unwatched.push(*id);
            }
        }
        for id in unwatched {
            inner.remove(&id);
        }
    }

    async fn with_game<F, T>(&self, id: GameId, f: F) -> Result<T, CardsError>
    where
        F: FnOnce(&mut Game) -> Result<T, CardsError>,
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
        participants: &HashSet<Participant>,
    ) -> Result<(), CardsError> {
        let mut participants = participants.iter().cloned().collect::<Vec<_>>();
        participants.shuffle(&mut rand::thread_rng());
        self.db.run_with_retry(|tx| {
            persist_events(
                &tx,
                id,
                0,
                &[
                    GameDbEvent::Sit {
                        north: participants[0].player.clone(),
                        east: participants[1].player.clone(),
                        south: participants[2].player.clone(),
                        west: participants[3].player.clone(),
                        rules: participants[0].rules,
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
        name: Name,
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        let (tx, rx) = unbounded_channel();
        self.with_game(id, |game| {
            let seat = game.seat(&name);
            let mut copy = Game::new();
            for db_event in &game.events {
                copy.apply(&db_event);
                for fe_event in copy.as_fe_events(db_event) {
                    send_event(seat, &tx, &fe_event);
                }
            }
            game.subscribers.insert(name, tx);
            Ok(())
        })
        .await?;
        Ok(rx)
    }

    pub async fn pass_cards(
        &self,
        id: GameId,
        name: &Name,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_pass(id, seat, cards)?;
                let mut db_events = vec![GameDbEvent::SendPass { from: seat, cards }];
                let sender = seat.pass_sender(game.pass_direction);
                if game.pass_direction != PassDirection::Keeper && game.has_passed(sender) {
                    db_events.push(GameDbEvent::RecvPass {
                        to: seat,
                        cards: game.pre_pass_hand[sender.idx()] - game.post_pass_hand[sender.idx()],
                    });
                }
                let receiver = seat.pass_receiver(game.pass_direction);
                if game.pass_direction != PassDirection::Keeper && game.has_passed(receiver) {
                    db_events.push(GameDbEvent::RecvPass {
                        to: receiver,
                        cards,
                    });
                }
                if game.pass_direction == PassDirection::Keeper
                    && Seat::VALUES
                        .iter()
                        .all(|s| *s == seat || game.has_passed(*s))
                {
                    let passes = Cards::ALL
                        - game.post_pass_hand[0]
                        - game.post_pass_hand[1]
                        - game.post_pass_hand[2]
                        - game.post_pass_hand[3]
                        | cards;
                    let mut passes = passes.into_iter().collect::<Vec<_>>();
                    passes.shuffle(&mut rand::thread_rng());
                    db_events.push(GameDbEvent::RecvPass {
                        to: Seat::North,
                        cards: passes[0..3].iter().cloned().collect(),
                    });
                    db_events.push(GameDbEvent::RecvPass {
                        to: Seat::East,
                        cards: passes[3..6].iter().cloned().collect(),
                    });
                    db_events.push(GameDbEvent::RecvPass {
                        to: Seat::South,
                        cards: passes[6..9].iter().cloned().collect(),
                    });
                    db_events.push(GameDbEvent::RecvPass {
                        to: Seat::West,
                        cards: passes[9..12].iter().cloned().collect(),
                    });
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &db_events)
                })?;
                for db_event in db_events {
                    game.apply(&db_event);
                    for fe_event in game.as_fe_events(&db_event) {
                        game.broadcast(&fe_event);
                    }
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(name.clone())),
        })
        .await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        name: &Name,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_charge(id, seat, cards)?;
                let db_event = GameDbEvent::Charge { seat, cards };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &[db_event.clone()])
                })?;
                game.apply(&db_event);
                for fe_event in game.as_fe_events(&db_event) {
                    game.broadcast(&fe_event);
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(name.clone())),
        })
        .await
    }

    pub async fn play_card(&self, id: GameId, name: &Name, card: Card) -> Result<bool, CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_play(id, seat, card)?;
                let mut db_events = vec![GameDbEvent::Play { seat, card }];
                if game.hand.played | card == Cards::ALL
                    && game.pass_direction != PassDirection::Keeper
                {
                    db_events.push(deal());
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &db_events)?;
                    if game.hand.played | card == Cards::ALL
                        && game.pass_direction == PassDirection::Keeper
                    {
                        tx.execute("INSERT INTO game (id) VALUES (?)", &[&id])?;
                    }
                    Ok(())
                })?;
                for db_event in db_events {
                    game.apply(&db_event);
                    for fe_event in game.as_fe_events(&db_event) {
                        game.broadcast(&fe_event);
                    }
                }
                Ok(game.state == GameState::Complete)
            }
            None => Err(CardsError::IllegalPlayer(name.clone())),
        })
        .await
    }
}

#[derive(Debug)]
struct Game {
    events: Vec<GameDbEvent>,
    subscribers: HashMap<Name, UnboundedSender<GameFeEvent>>,
    players: [Name; 4],
    rules: ChargingRules,

    state: GameState,
    pre_pass_hand: [Cards; 4],

    pass_direction: PassDirection,
    post_pass_hand: [Cards; 4],

    charges: ChargeState,
    hand: HandState,
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
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],

            charges: ChargeState::new(ChargingRules::Classic, PassDirection::Left),
            hand: HandState::new(Seat::North),
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

    fn broadcast(&mut self, event: &GameFeEvent) {
        let players = self.players.clone();
        self.subscribers.retain(|name, tx| {
            let seat = seat(&players, name);
            send_event(seat, tx, event)
        });
    }

    fn seat(&self, name: &Name) -> Option<Seat> {
        seat(&self.players, name)
    }

    fn has_passed(&self, seat: Seat) -> bool {
        self.pre_pass_hand[seat.idx()] != self.post_pass_hand[seat.idx()]
    }

    fn apply(&mut self, event: &GameDbEvent) {
        match &event {
            GameDbEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
            } => {
                self.players = [
                    north.name().clone(),
                    east.name().clone(),
                    south.name().clone(),
                    west.name().clone(),
                ];
                self.rules = *rules;
            }
            GameDbEvent::Deal {
                north,
                east,
                south,
                west,
            } => {
                self.pre_pass_hand[Seat::North.idx()] = *north;
                self.post_pass_hand[Seat::North.idx()] = *north;
                self.pre_pass_hand[Seat::East.idx()] = *east;
                self.post_pass_hand[Seat::East.idx()] = *east;
                self.pre_pass_hand[Seat::South.idx()] = *south;
                self.post_pass_hand[Seat::South.idx()] = *south;
                self.pre_pass_hand[Seat::West.idx()] = *west;
                self.post_pass_hand[Seat::West.idx()] = *west;
                self.charges = ChargeState::new(self.rules, self.pass_direction);
                self.state = if self.pass_direction == PassDirection::Keeper {
                    GameState::KeeperCharging
                } else {
                    GameState::Passing
                };
            }
            GameDbEvent::SendPass { from, cards } => {
                self.post_pass_hand[from.idx()] -= *cards;
            }
            GameDbEvent::RecvPass { to, cards } => {
                self.post_pass_hand[to.idx()] |= *cards;
                if Seat::VALUES.iter().all(|seat| self.has_passed(*seat)) {
                    self.state = GameState::Charging;
                }
            }
            GameDbEvent::Charge { seat, cards } => {
                let done = self.charges.charge(*seat, *cards);
                if done {
                    match self.state {
                        GameState::KeeperCharging => {
                            if self.charges.charged.is_empty() {
                                self.state = GameState::Passing;
                            } else {
                                self.state = GameState::Playing;
                                self.hand.reset(self.owner(Card::TwoClubs));
                            }
                        }
                        GameState::Charging => {
                            self.state = GameState::Playing;
                            self.hand.reset(self.owner(Card::TwoClubs));
                        }
                        _ => unreachable!(),
                    }
                }
            }
            GameDbEvent::Play { seat: _, card } => {
                self.hand.play(*card);
                if self.hand.played == Cards::ALL {
                    match self.pass_direction.next() {
                        Some(pass_direction) => {
                            self.pass_direction = pass_direction;
                        }
                        None => self.state = GameState::Complete,
                    }
                }
            }
        }
        self.events.push(event.clone());
    }

    fn verify_pass(&self, id: GameId, seat: Seat, cards: Cards) -> Result<(), CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        if self.state != GameState::Passing {
            return Err(CardsError::IllegalAction(self.state));
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

    fn verify_charge(&mut self, id: GameId, seat: Seat, cards: Cards) -> Result<(), CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        let hand_cards = match self.state {
            GameState::KeeperCharging | GameState::Charging => self.post_pass_hand[seat.idx()],
            _ => return Err(CardsError::IllegalAction(self.state)),
        };
        if !hand_cards.contains_all(cards) {
            return Err(CardsError::NotYourCards(cards - hand_cards));
        }
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(CardsError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        if self.charges.charged.contains_any(cards) {
            return Err(CardsError::AlreadyCharged(self.charges.charged & cards));
        }
        match self.charges.next_charger {
            Some(s) if s != seat => {
                return Err(CardsError::NotYourTurn(self.players[s.idx()].clone()))
            }
            _ => {}
        }
        Ok(())
    }

    fn verify_play(&mut self, id: GameId, seat: Seat, card: Card) -> Result<(), CardsError> {
        if self.state == GameState::Complete {
            return Err(CardsError::GameComplete(id));
        }
        if self.state != GameState::Playing {
            return Err(CardsError::IllegalAction(self.state));
        }
        let mut plays = self.post_pass_hand[seat.idx()] - self.hand.played;
        if !plays.contains(card) {
            return Err(CardsError::NotYourCards(card.into()));
        }
        if seat != self.hand.next_player {
            return Err(CardsError::NotYourTurn(
                self.players[self.hand.next_player.idx()].clone(),
            ));
        }
        if self.hand.led_suits.is_empty() {
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
        if !self.hand.current_trick.is_empty() {
            let suit = self.hand.current_trick[0].suit();
            if suit.cards().contains_any(plays) {
                plays &= suit.cards();
                if !plays.contains(card) {
                    return Err(CardsError::MustFollowSuit);
                }
                if !self.hand.led_suits.contains_any(suit.cards()) && plays.len() > 1 {
                    plays -= self.charges.charged;
                    if !plays.contains(card) {
                        return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                    }
                }
            }
        } else {
            if !self.hand.led_suits.contains_any(Cards::HEARTS)
                && !Cards::HEARTS.contains_all(plays)
                && card.suit() == Suit::Hearts
            {
                plays -= Cards::HEARTS;
                if !plays.contains(card) {
                    return Err(CardsError::HeartsNotBroken);
                }
            }
            let unled_charges = self.charges.charged - self.hand.led_suits;
            if !unled_charges.contains_all(plays) {
                plays -= unled_charges;
                if !plays.contains(card) {
                    return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                }
            }
        }
        Ok(())
    }

    fn as_fe_events(&self, event: &GameDbEvent) -> Vec<GameFeEvent> {
        match event {
            GameDbEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
            } => vec![GameFeEvent::Sit {
                north: north.clone(),
                east: east.clone(),
                south: south.clone(),
                west: west.clone(),
                rules: *rules,
            }],
            GameDbEvent::Deal {
                north,
                east,
                south,
                west,
            } => {
                let mut events = vec![GameFeEvent::Deal {
                    north: *north,
                    east: *east,
                    south: *south,
                    west: *west,
                    pass: self.pass_direction,
                }];
                if self.state == GameState::KeeperCharging {
                    events.push(GameFeEvent::StartCharging {
                        seat: self.charges.next_charger,
                    });
                } else if self.state == GameState::Passing {
                    events.push(GameFeEvent::StartPassing);
                }
                events
            }
            GameDbEvent::SendPass { from, cards } => vec![GameFeEvent::SendPass {
                from: *from,
                cards: *cards,
            }],
            GameDbEvent::RecvPass { to, cards } => {
                let mut events = vec![GameFeEvent::RecvPass {
                    to: *to,
                    cards: *cards,
                }];
                if self.state == GameState::Charging {
                    events.push(GameFeEvent::StartCharging {
                        seat: self.charges.next_charger,
                    });
                }
                events
            }
            GameDbEvent::Charge { seat, cards } => {
                let mut events = vec![];
                if self.rules.blind() {
                    events.push(GameFeEvent::BlindCharge {
                        seat: *seat,
                        count: cards.len(),
                    });
                } else {
                    events.push(GameFeEvent::Charge {
                        seat: *seat,
                        cards: *cards,
                    });
                }
                if self.charges.all_done_charging() {
                    if self.rules.blind() {
                        for seat in &Seat::VALUES {
                            events.push(GameFeEvent::RevealCharge {
                                seat: *seat,
                                cards: self.charges.charged & self.post_pass_hand[seat.idx()],
                            });
                        }
                    }
                    if self.state == GameState::Playing {
                        events.push(GameFeEvent::StartTrick {
                            leader: self.owner(Card::TwoClubs),
                            trick_number: self.hand.trick_number,
                        });
                    } else if self.state == GameState::Passing {
                        events.push(GameFeEvent::StartPassing);
                    }
                }
                events
            }
            GameDbEvent::Play { seat, card } => {
                let trick_number = if self.hand.current_trick.is_empty() {
                    self.hand.trick_number - 1
                } else {
                    self.hand.trick_number
                };
                let mut events = vec![GameFeEvent::Play {
                    seat: *seat,
                    card: *card,
                    trick_number,
                }];
                if self.hand.current_trick.is_empty() && self.hand.played != Cards::ALL {
                    events.push(GameFeEvent::StartTrick {
                        leader: self.hand.next_player,
                        trick_number: self.hand.trick_number,
                    });
                }
                events
            }
        }
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameDbEvent {
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
    SendPass {
        from: Seat,
        cards: Cards,
    },
    RecvPass {
        to: Seat,
        cards: Cards,
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

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameFeEvent {
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
        pass: PassDirection,
    },
    StartPassing,
    SendPass {
        from: Seat,
        cards: Cards,
    },
    RecvPass {
        to: Seat,
        cards: Cards,
    },
    StartCharging {
        seat: Option<Seat>,
    },
    BlindCharge {
        seat: Seat,
        count: usize,
    },
    Charge {
        seat: Seat,
        cards: Cards,
    },
    RevealCharge {
        seat: Seat,
        cards: Cards,
    },
    StartTrick {
        leader: Seat,
        trick_number: usize,
    },
    Play {
        seat: Seat,
        card: Card,
        trick_number: usize,
    },
}

impl Event for GameFeEvent {
    fn is_ping(&self) -> bool {
        match self {
            GameFeEvent::Ping => true,
            _ => false,
        }
    }
}

impl ToSql for GameDbEvent {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(self).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl FromSql for GameDbEvent {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| serde_json::from_str(s).unwrap())
    }
}

fn deal() -> GameDbEvent {
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    GameDbEvent::Deal {
        north: deck[0..13].iter().cloned().collect(),
        east: deck[13..26].iter().cloned().collect(),
        south: deck[26..39].iter().cloned().collect(),
        west: deck[39..52].iter().cloned().collect(),
    }
}

fn seat(players: &[Name; 4], name: &Name) -> Option<Seat> {
    players
        .iter()
        .position(|p| p == name)
        .map(|idx| Seat::VALUES[idx])
}

pub fn persist_events(
    tx: &Transaction,
    id: GameId,
    mut event_id: EventId,
    events: &[GameDbEvent],
) -> Result<(), CardsError> {
    let mut stmt =
        tx.prepare("INSERT INTO event (game_id, event_id, timestamp, event) VALUES (?, ?, ?, ?)")?;
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
        game.apply(&serde_json::from_str(&row.get::<_, String>(0)?)?);
    }
    Ok(())
}

fn send_event(seat: Option<Seat>, tx: &UnboundedSender<GameFeEvent>, event: &GameFeEvent) -> bool {
    let event = match event {
        GameFeEvent::Ping
        | GameFeEvent::Play { .. }
        | GameFeEvent::Sit { .. }
        | GameFeEvent::Charge { .. }
        | GameFeEvent::BlindCharge { .. }
        | GameFeEvent::RevealCharge { .. }
        | GameFeEvent::StartPassing
        | GameFeEvent::StartCharging { .. }
        | GameFeEvent::StartTrick { .. } => event.clone(),
        GameFeEvent::Deal {
            north,
            east,
            south,
            west,
            pass,
        } => match seat {
            None => event.clone(),
            Some(seat) => match seat {
                Seat::North => GameFeEvent::Deal {
                    north: *north,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Seat::East => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: *east,
                    south: Cards::NONE,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Seat::South => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: *south,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Seat::West => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: *west,
                    pass: *pass,
                },
            },
        },
        GameFeEvent::SendPass { from, cards: _ } => match seat {
            Some(seat) if seat != *from => GameFeEvent::SendPass {
                from: *from,
                cards: Cards::NONE,
            },
            _ => event.clone(),
        },
        GameFeEvent::RecvPass { to, cards: _ } => match seat {
            Some(seat) if seat != *to => GameFeEvent::RecvPass {
                to: *to,
                cards: Cards::NONE,
            },
            _ => event.clone(),
        },
    };
    tx.send(event).is_ok()
}
