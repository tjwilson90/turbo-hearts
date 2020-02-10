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
                    GameDbEvent::Sit {
                        north: order[0].clone(),
                        east: order[1].clone(),
                        south: order[2].clone(),
                        west: order[3].clone(),
                        rules: players[order[0]],
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
    ) -> Result<UnboundedReceiver<GameFeEvent>, CardsError> {
        let (tx, rx) = unbounded_channel();
        self.with_game(id, |game| {
            let seat = game.seat(&player);
            let mut copy = Game::new();
            for db_event in &game.events {
                for fe_event in as_fe_events(&copy, db_event) {
                    send_event(seat, &tx, &fe_event);
                }
                copy.apply(db_event.clone());
            }
            game.subscribers.insert(player, tx);
            Ok(())
        })
        .await?;
        Ok(rx)
    }

    pub async fn pass_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| match game.seat(&player) {
            Some(seat) => {
                game.verify_pass(id, seat, cards)?;
                let db_event = GameDbEvent::Pass { from: seat, cards };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &[db_event.clone()])
                })?;
                for fe_event in as_fe_events(&game, &db_event) {
                    game.broadcast(&fe_event);
                }
                game.apply(db_event);
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
                game.verify_charge(id, seat, cards)?;
                let db_event = GameDbEvent::Charge { seat, cards };
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &[db_event.clone()])
                })?;
                for fe_event in as_fe_events(&game, &db_event) {
                    game.broadcast(&fe_event);
                }
                game.apply(db_event);
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
    ) -> Result<bool, CardsError> {
        self.with_game(id, |game| match game.seat(&player) {
            Some(seat) => {
                game.verify_play(id, seat, card)?;
                let mut db_events = vec![GameDbEvent::Play { seat, card }];
                if game.played | card == Cards::ALL && game.pass_direction != PassDirection::Keeper
                {
                    db_events.push(deal());
                }
                self.db.run_with_retry(|tx| {
                    persist_events(&tx, id, game.events.len() as u32, &db_events)?;
                    if game.played | card == Cards::ALL
                        && game.pass_direction == PassDirection::Keeper
                    {
                        tx.execute("INSERT INTO game (id) VALUES (?)", &[&id])?;
                    }
                    Ok(())
                })?;
                for db_event in db_events {
                    for fe_event in as_fe_events(&game, &db_event) {
                        game.broadcast(&fe_event);
                    }
                    game.apply(db_event);
                }
                Ok(game.state == GameState::Complete)
            }
            None => Err(CardsError::IllegalPlayer(player)),
        })
        .await
    }
}

#[derive(Debug)]
struct Game {
    events: Vec<GameDbEvent>,
    subscribers: HashMap<Player, UnboundedSender<GameFeEvent>>,
    players: [Player; 4],
    rules: ChargingRules,

    state: GameState,
    pre_pass_hand: [Cards; 4],

    pass_direction: PassDirection,
    post_pass_hand: [Cards; 4],

    next_charger: Option<Seat>,
    done_charging: [bool; 4],
    charges: Cards,

    leads: Cards,
    played: Cards,
    won: [Cards; 4],
    trick: Trick,
    trick_number: usize,
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

            next_charger: None,
            done_charging: [false, false, false, false],
            charges: Cards::NONE,

            leads: Cards::NONE,
            played: Cards::NONE,
            won: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            trick: Trick::new(Seat::North),
            trick_number: 0,
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
        self.subscribers.retain(|p, tx| {
            let seat = seat(&players, p);
            send_event(seat, tx, event)
        });
    }

    fn seat(&self, player: &Player) -> Option<Seat> {
        seat(&self.players, player)
    }

    fn has_passed(&self, seat: Seat) -> bool {
        self.pre_pass_hand[seat.idx()] != self.post_pass_hand[seat.idx()]
    }

    fn apply(&mut self, event: GameDbEvent) {
        match &event {
            GameDbEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
            } => {
                self.players = [north.clone(), east.clone(), south.clone(), west.clone()];
                self.rules = *rules;
                if !rules.free() {
                    self.next_charger = Some(Seat::North);
                }
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
            }
            GameDbEvent::Pass { from, cards } => {
                self.post_pass_hand[from.idx()] -= *cards;
                let to = from.pass_receiver(self.pass_direction);
                self.post_pass_hand[to.idx()] |= *cards;
                if self
                    .pre_pass_hand
                    .iter()
                    .zip(self.post_pass_hand.iter())
                    .all(|(pre, post)| !(*pre - *post).is_empty())
                {
                    self.state = GameState::Charging;
                }
            }
            GameDbEvent::Charge { seat, cards } => {
                self.charges |= *cards;
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
            GameDbEvent::Play { seat: _, card } => {
                self.played |= *card;
                if self.trick.cards.is_empty() {
                    self.leads |= *card;
                }
                self.trick.play(*card);
                if let Some(winning_card) = self.trick.winner() {
                    let winner = self.owner(winning_card);
                    self.won[winner.idx()] |= self.trick.cards;
                    self.trick = Trick::new(winner);
                    self.trick_number += 1;
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
        if self.charges.contains_any(cards) {
            return Err(CardsError::AlreadyCharged(self.charges & cards));
        }
        match self.next_charger {
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
            let suit = self.trick.lead.unwrap().cards();
            if !suit.contains(card) && current_hand.contains_any(suit) {
                return Err(CardsError::MustFollowSuit);
            }
            if self.trick.cards.contains(Card::TwoClubs)
                && Cards::POINTS.contains(card)
                && !Cards::POINTS.contains_all(current_hand)
            {
                return Err(CardsError::NoPointsOnFirstTrick);
            }
        }
        if (self.trick.cards.is_empty() || self.trick.lead.unwrap().cards().contains(card))
            && Cards::CHARGEABLE.contains(card)
            && !self.leads.contains_any(card.suit().cards())
            && (current_hand & card.suit().cards()).len() > 1
        {
            return Err(CardsError::NoChargeOnFirstTrickOfSuit);
        }
        Ok(())
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
    Pass {
        from: Seat,
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

fn seat(players: &[Player; 4], player: &Player) -> Option<Seat> {
    players
        .iter()
        .position(|p| p == player)
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
        game.apply(serde_json::from_str(&row.get::<_, String>(0)?)?);
    }
    Ok(())
}

fn as_fe_events(game: &Game, event: &GameDbEvent) -> Vec<GameFeEvent> {
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
        } => vec![GameFeEvent::Deal {
            north: *north,
            east: *east,
            south: *south,
            west: *west,
        }],
        GameDbEvent::Pass { from, cards } => {
            let mut events = vec![GameFeEvent::SendPass {
                from: *from,
                cards: *cards,
            }];
            let sender = from.pass_sender(game.pass_direction);
            if game.has_passed(sender) {
                events.push(GameFeEvent::RecvPass {
                    to: *from,
                    cards: game.pre_pass_hand[sender.idx()] - game.post_pass_hand[sender.idx()],
                });
            }
            let receiver = from.pass_receiver(game.pass_direction);
            if game.has_passed(receiver) {
                events.push(GameFeEvent::RecvPass {
                    to: receiver,
                    cards: *cards,
                });
            }
            events
        }
        GameDbEvent::Charge { seat, cards } => {
            let mut events = vec![];
            if game.rules.blind() {
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
            if game.rules.blind()
                && cards.is_empty()
                && game.done_charging[seat.next().idx()]
                && game.done_charging[seat.next().next().idx()]
                && game.done_charging[seat.next().next().next().idx()]
            {
                for seat in &Seat::VALUES {
                    events.push(GameFeEvent::Charge {
                        seat: *seat,
                        cards: game.charges & game.post_pass_hand[seat.idx()],
                    });
                }
            }
            events
        }
        GameDbEvent::Play { seat, card } => vec![GameFeEvent::Play {
            seat: *seat,
            card: *card,
            trick_number: game.trick_number,
        }],
    }
}

fn send_event(seat: Option<Seat>, tx: &UnboundedSender<GameFeEvent>, event: &GameFeEvent) -> bool {
    let event = match event {
        GameFeEvent::Ping
        | GameFeEvent::Play { .. }
        | GameFeEvent::Sit { .. }
        | GameFeEvent::Charge { .. }
        | GameFeEvent::BlindCharge { .. } => event.clone(),
        GameFeEvent::Deal {
            north,
            east,
            south,
            west,
        } => match seat {
            None => event.clone(),
            Some(seat) => match seat {
                Seat::North => GameFeEvent::Deal {
                    north: *north,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: Cards::NONE,
                },
                Seat::East => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: *east,
                    south: Cards::NONE,
                    west: Cards::NONE,
                },
                Seat::South => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: *south,
                    west: Cards::NONE,
                },
                Seat::West => GameFeEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: *west,
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
