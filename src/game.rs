use crate::cards::{GamePhase, GameState};
use crate::types::Participant;
use crate::{
    cards::{Card, Cards, Suit},
    db::Database,
    error::CardsError,
    hacks::{unbounded_channel, Mutex, UnboundedReceiver, UnboundedSender},
    types::{ChargingRules, Event, GameId, Player, Seat},
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
            if game.subscribers.is_empty() || game.state.is_complete() {
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
                    GameBeEvent::Sit {
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
        name: String,
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        let (tx, rx) = unbounded_channel();
        self.with_game(id, |game| {
            let seat = game.seat(&name);
            let mut copy = Game::new();
            for event in &game.events {
                copy.apply(&event);
                send_event(seat, &tx, copy.state.rules, &event);
            }
            if !game.state.is_complete() {
                game.subscribers.insert(name.to_string(), tx);
            }
            Ok(())
        })
        .await?;
        Ok(rx)
    }

    pub async fn pass_cards(&self, id: GameId, name: &str, cards: Cards) -> Result<(), CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_pass(id, seat, cards)?;
                let mut db_events = vec![GameBeEvent::SendPass { from: seat, cards }];
                if game.state.phase != GamePhase::PassKeeper {
                    let sender = game.state.phase.pass_sender(seat);
                    if game.state.sent_pass[sender.idx()] {
                        db_events.push(GameBeEvent::RecvPass {
                            to: seat,
                            cards: game.pre_pass_hand[sender.idx()]
                                - game.post_pass_hand[sender.idx()],
                        });
                    }
                    let receiver = game.state.phase.pass_receiver(seat);
                    if game.state.sent_pass[receiver.idx()] {
                        db_events.push(GameBeEvent::RecvPass {
                            to: receiver,
                            cards,
                        });
                    }
                }
                if game.state.phase == GamePhase::PassKeeper
                    && Seat::all(|s| s == seat || game.state.sent_pass[s.idx()])
                {
                    let passes = Cards::ALL
                        - game.post_pass_hand[0]
                        - game.post_pass_hand[1]
                        - game.post_pass_hand[2]
                        - game.post_pass_hand[3]
                        | cards;
                    let mut passes = passes.into_iter().collect::<Vec<_>>();
                    passes.shuffle(&mut rand::thread_rng());
                    db_events.push(GameBeEvent::RecvPass {
                        to: Seat::North,
                        cards: passes[0..3].iter().cloned().collect(),
                    });
                    db_events.push(GameBeEvent::RecvPass {
                        to: Seat::East,
                        cards: passes[3..6].iter().cloned().collect(),
                    });
                    db_events.push(GameBeEvent::RecvPass {
                        to: Seat::South,
                        cards: passes[6..9].iter().cloned().collect(),
                    });
                    db_events.push(GameBeEvent::RecvPass {
                        to: Seat::West,
                        cards: passes[9..12].iter().cloned().collect(),
                    });
                }
                self.db
                    .run_with_retry(|tx| persist_events(&tx, id, game.events.len(), &db_events))?;
                for db_event in db_events {
                    for fe_event in game.as_fe_events(&db_event) {
                        game.broadcast(&fe_event);
                        game.apply(&fe_event);
                    }
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(name.to_string())),
        })
        .await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        name: &str,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_charge(id, seat, cards)?;
                let db_event = GameBeEvent::Charge { seat, cards };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len(), &[db_event.clone()])
                })?;
                for fe_event in game.as_fe_events(&db_event) {
                    game.broadcast(&fe_event);
                    game.apply(&fe_event);
                }
                Ok(())
            }
            None => Err(CardsError::IllegalPlayer(name.to_string())),
        })
        .await
    }

    pub async fn play_card(&self, id: GameId, name: &str, card: Card) -> Result<bool, CardsError> {
        self.with_game(id, |game| match game.seat(&name) {
            Some(seat) => {
                game.verify_play(id, seat, card)?;
                let mut db_events = vec![GameBeEvent::Play { seat, card }];
                if game.state.played | card == Cards::ALL
                    && game.state.phase != GamePhase::PlayKeeper
                {
                    db_events.push(deal());
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len(), &db_events)?;
                    if game.state.played | card == Cards::ALL
                        && game.state.phase == GamePhase::PlayKeeper
                    {
                        tx.execute("INSERT INTO game (id) VALUES (?)", &[&id])?;
                    }
                    Ok(())
                })?;
                for db_event in db_events {
                    for fe_event in game.as_fe_events(&db_event) {
                        game.broadcast(&fe_event);
                        game.apply(&fe_event);
                    }
                }
                Ok(game.state.phase == GamePhase::Complete)
            }
            None => Err(CardsError::IllegalPlayer(name.to_string())),
        })
        .await
    }
}

#[derive(Debug)]
struct Game {
    events: Vec<GameFeEvent>,
    subscribers: HashMap<String, UnboundedSender<GameFeEvent>>,
    pre_pass_hand: [Cards; 4],
    post_pass_hand: [Cards; 4],
    state: GameState,
}

impl Game {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            subscribers: HashMap::new(),
            pre_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            state: GameState::new(),
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
        let players = &self.state.players;
        let rules = self.state.rules;
        self.subscribers.retain(|name, tx| {
            let seat = seat(&players, name);
            send_event(seat, tx, rules, event)
        });
    }

    fn seat(&self, name: &str) -> Option<Seat> {
        seat(&self.state.players, name)
    }

    fn apply(&mut self, event: &GameFeEvent) {
        self.state.apply(&event);
        match &event {
            GameFeEvent::Deal {
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
            }
            GameFeEvent::SendPass { from, cards } => {
                self.post_pass_hand[from.idx()] -= *cards;
            }
            GameFeEvent::RecvPass { to, cards } => {
                self.post_pass_hand[to.idx()] |= *cards;
            }
            GameFeEvent::Charge { .. } | GameFeEvent::RevealCharges { .. } => {
                if self.state.phase.playing() {
                    self.state.next_player = Some(self.owner(Card::TwoClubs));
                }
            }
            _ => {}
        }
        self.events.push(event.clone());
    }

    fn verify_pass(&self, id: GameId, seat: Seat, cards: Cards) -> Result<(), CardsError> {
        if self.state.is_complete() {
            return Err(CardsError::GameComplete(id));
        }
        if !self.state.phase.passing() {
            return Err(CardsError::IllegalAction(self.state.phase));
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
        if self.state.is_complete() {
            return Err(CardsError::GameComplete(id));
        }
        if !self.state.phase.charging() {
            return Err(CardsError::IllegalAction(self.state.phase));
        }
        let hand_cards = self.post_pass_hand[seat.idx()];
        if !hand_cards.contains_all(cards) {
            return Err(CardsError::NotYourCards(cards - hand_cards));
        }
        if !Cards::CHARGEABLE.contains_all(cards) {
            return Err(CardsError::Unchargeable(cards - Cards::CHARGEABLE));
        }
        if self.state.charged[seat.idx()].contains_any(cards) {
            return Err(CardsError::AlreadyCharged(
                self.state.charged[seat.idx()] & cards,
            ));
        }
        if !self.state.can_charge(seat) {
            return Err(CardsError::NotYourTurn(
                self.state.players[self.state.next_charger.unwrap().idx()].clone(),
            ));
        }
        Ok(())
    }

    fn verify_play(&mut self, id: GameId, seat: Seat, card: Card) -> Result<(), CardsError> {
        if self.state.is_complete() {
            return Err(CardsError::GameComplete(id));
        }
        if !self.state.phase.playing() {
            return Err(CardsError::IllegalAction(self.state.phase));
        }
        let mut plays = self.post_pass_hand[seat.idx()] - self.state.played;
        if !plays.contains(card) {
            return Err(CardsError::NotYourCards(card.into()));
        }
        if seat != self.state.next_player.unwrap() {
            return Err(CardsError::NotYourTurn(
                self.state.players[self.state.next_player.unwrap().idx()].clone(),
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
            let suit = self.state.current_trick[0].suit();
            if suit.cards().contains_any(plays) {
                plays &= suit.cards();
                if !plays.contains(card) {
                    return Err(CardsError::MustFollowSuit);
                }
                if !self.state.led_suits.contains_any(suit.cards()) && plays.len() > 1 {
                    plays -= self.state.charged[seat.idx()];
                    if !plays.contains(card) {
                        return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                    }
                }
            }
        } else {
            if !self.state.played.contains_any(Cards::HEARTS)
                && !Cards::HEARTS.contains_all(plays)
                && card.suit() == Suit::Hearts
            {
                plays -= Cards::HEARTS;
                if !plays.contains(card) {
                    return Err(CardsError::HeartsNotBroken);
                }
            }
            let unled_charges = self.state.charged[seat.idx()] - self.state.led_suits;
            if !unled_charges.contains_all(plays) {
                plays -= unled_charges;
                if !plays.contains(card) {
                    return Err(CardsError::NoChargeOnFirstTrickOfSuit);
                }
            }
        }
        Ok(())
    }

    fn as_fe_events(&self, event: &GameBeEvent) -> Vec<GameFeEvent> {
        match event {
            GameBeEvent::Sit {
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
            GameBeEvent::Deal {
                north,
                east,
                south,
                west,
            } => vec![GameFeEvent::Deal {
                north: *north,
                east: *east,
                south: *south,
                west: *west,
            }],
            GameBeEvent::SendPass { from, cards } => vec![GameFeEvent::SendPass {
                from: *from,
                cards: *cards,
            }],
            GameBeEvent::RecvPass { to, cards } => vec![GameFeEvent::RecvPass {
                to: *to,
                cards: *cards,
            }],
            GameBeEvent::Charge { seat, cards } => {
                let mut events = vec![GameFeEvent::Charge {
                    seat: *seat,
                    cards: *cards,
                }];
                if cards.is_empty()
                    && self.state.rules.blind()
                    && self.state.done_charging[seat.left().idx()]
                    && self.state.done_charging[seat.across().idx()]
                    && self.state.done_charging[seat.right().idx()]
                {
                    events.push(GameFeEvent::RevealCharges {
                        north: self.state.charged[0],
                        east: self.state.charged[1],
                        south: self.state.charged[2],
                        west: self.state.charged[3],
                    });
                }
                events
            }
            GameBeEvent::Play { seat, card } => vec![GameFeEvent::Play {
                seat: *seat,
                card: *card,
            }],
        }
    }
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameBeEvent {
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
    },
    SendPass {
        from: Seat,
        cards: Cards,
    },
    RecvPass {
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
    RevealCharges {
        north: Cards,
        east: Cards,
        south: Cards,
        west: Cards,
    },
    Play {
        seat: Seat,
        card: Card,
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

impl ToSql for GameBeEvent {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        let json = serde_json::to_string(self).unwrap();
        Ok(ToSqlOutput::Owned(Value::Text(json)))
    }
}

impl FromSql for GameBeEvent {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| serde_json::from_str(s).unwrap())
    }
}

fn deal() -> GameBeEvent {
    let mut deck = Cards::ALL.into_iter().collect::<Vec<_>>();
    deck.shuffle(&mut rand::thread_rng());
    GameBeEvent::Deal {
        north: deck[0..13].iter().cloned().collect(),
        east: deck[13..26].iter().cloned().collect(),
        south: deck[26..39].iter().cloned().collect(),
        west: deck[39..52].iter().cloned().collect(),
    }
}

fn seat(players: &[String; 4], name: &str) -> Option<Seat> {
    players
        .iter()
        .position(|p| p == name)
        .map(|idx| Seat::VALUES[idx])
}

pub fn persist_events(
    tx: &Transaction,
    id: GameId,
    event_id: usize,
    events: &[GameBeEvent],
) -> Result<(), CardsError> {
    let mut stmt =
        tx.prepare("INSERT INTO event (game_id, event_id, timestamp, event) VALUES (?, ?, ?, ?)")?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let mut event_id = event_id as isize;
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

fn send_event(
    seat: Option<Seat>,
    tx: &UnboundedSender<GameFeEvent>,
    rules: ChargingRules,
    event: &GameFeEvent,
) -> bool {
    let event = match event {
        GameFeEvent::Ping
        | GameFeEvent::Play { .. }
        | GameFeEvent::Sit { .. }
        | GameFeEvent::BlindCharge { .. }
        | GameFeEvent::RevealCharges { .. } => event.clone(),
        GameFeEvent::Deal {
            north,
            east,
            south,
            west,
        } => match seat {
            Some(Seat::North) => GameFeEvent::Deal {
                north: *north,
                east: Cards::NONE,
                south: Cards::NONE,
                west: Cards::NONE,
            },
            Some(Seat::East) => GameFeEvent::Deal {
                north: Cards::NONE,
                east: *east,
                south: Cards::NONE,
                west: Cards::NONE,
            },
            Some(Seat::South) => GameFeEvent::Deal {
                north: Cards::NONE,
                east: Cards::NONE,
                south: *south,
                west: Cards::NONE,
            },
            Some(Seat::West) => GameFeEvent::Deal {
                north: Cards::NONE,
                east: Cards::NONE,
                south: Cards::NONE,
                west: *west,
            },
            None => event.clone(),
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
        GameFeEvent::Charge { seat: s, cards } => match seat {
            Some(seat) if *s != seat && rules.blind() => GameFeEvent::BlindCharge {
                seat: *s,
                count: cards.len(),
            },
            _ => event.clone(),
        },
    };
    tx.send(event).is_ok()
}
