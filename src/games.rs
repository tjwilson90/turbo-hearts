use crate::{
    cards::{Card, Cards, Suit},
    db::Database,
    error::CardsError,
    hacks::{Mutex, UnboundedSender},
    types::{ChargingRules, EventId, GameId, PassDirection, Player, Seat},
};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    OptionalExtension, ToSql,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

#[derive(Clone)]
pub struct Games {
    db: Database,
    inner: Arc<Mutex<HashMap<GameId, Arc<Mutex<Option<Game>>>>>>,
}

impl Games {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            inner: Arc::new(Mutex::new(HashMap::new())),
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
                    let game = Arc::new(Mutex::new(None));
                    entry.insert(game.clone());
                    game
                }
            }
        };
        let mut game = game.lock().await;
        if game.is_none() {
            *game = Some(self.load_game(id)?)
        }
        f(game.as_mut().unwrap())
    }

    fn load_game(&self, id: GameId) -> Result<Game, CardsError> {
        self.db.run_read_only(|tx| {
            let mut game = tx
                .query_row(
                    "SELECT north, east, south, west, rules FROM game WHERE id = ?",
                    &[&id],
                    |row| {
                        Ok(Game::new(
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
                .ok_or(CardsError::UnknownGame(id))?;
            let mut stmt = tx.prepare(
                "SELECT event_id, hand, seat, timestamp, kind FROM event WHERE game_id = ?",
            )?;
            let mut rows = stmt.query(&[&id])?;
            while let Some(row) = rows.next()? {
                game.apply(Event {
                    id: row.get(0)?,
                    hand: row.get(1)?,
                    seat: row.get(2)?,
                    timestamp: row.get(3)?,
                    kind: serde_json::from_str(&row.get::<_, String>(4)?)?,
                });
            }
            Ok(game)
        })
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
        self.with_game(id, |game| game.pass_cards(player, cards))
            .await
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| game.charge_cards(player, cards))
            .await
    }

    pub async fn play_card(
        &self,
        id: GameId,
        player: Player,
        card: Card,
    ) -> Result<(), CardsError> {
        self.with_game(id, |game| game.play_card(player, card))
            .await
    }
}

struct Game {
    id: GameId,
    rules: ChargingRules,
    subscribers: HashMap<Player, UnboundedSender<Event>>,
    players: [Player; 4],
    hands: [Hand; 4],
    events: Vec<Event>,
}

struct Hand {
    state: HandState,
    pre_pass_hand: [Cards; 4],

    received_pass: [Cards; 4],
    post_pass_hand: [Cards; 4],

    done_charging: [bool; 4],
    charges: Cards,

    played: Cards,
    won: [Cards; 4],
    trick: Trick,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandState {
    KeeperCharging,
    Passing,
    Charging,
    Playing,
    Complete,
}

impl Game {
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
                Hand::new(PassDirection::Left),
                Hand::new(PassDirection::Right),
                Hand::new(PassDirection::Across),
                Hand::new(PassDirection::Keeper),
            ],
            events: Vec::new(),
        }
    }

    fn apply(&mut self, event: Event) {
        let hand: &mut Hand = &mut self.hands[event.hand.idx()];
        let seat = event.seat.idx();
        match event.kind {
            EventKind::Ping => unreachable!(),
            EventKind::ReceiveHand(cards) => {
                hand.pre_pass_hand[seat] = cards;
                hand.post_pass_hand[seat] = cards;
            }
            EventKind::ReceivePass(cards) => {
                hand.received_pass[seat] = cards;
                if cards.contains(Card::TwoClubs) {
                    hand.trick = Trick::new(event.seat);
                }
                for post_pass_hand in &mut hand.post_pass_hand {
                    *post_pass_hand -= cards;
                }
                hand.post_pass_hand[seat] |= cards;
                if hand.received_pass.iter().all(|pass| pass.len() > 0) {
                    hand.state = HandState::Charging;
                }
            }
            EventKind::BlindCharge(_) => unreachable!(),
            EventKind::ChargeCards(cards) => {
                hand.charges |= cards;
                let chain =
                    self.rules == ChargingRules::Chain || self.rules == ChargingRules::BlindChain;
                if cards.is_empty() {
                    hand.done_charging[seat] = true;
                } else {
                    for done_charging in &mut hand.done_charging {
                        *done_charging = false;
                    }
                    hand.done_charging[seat] = !chain;
                }
                if hand.done_charging.iter().all(|done| *done) {
                    match hand.state {
                        HandState::KeeperCharging => {
                            hand.state = if hand.charges.is_empty() {
                                HandState::Passing
                            } else {
                                HandState::Playing
                            };
                        }
                        HandState::Charging => {
                            hand.state = HandState::Playing;
                        }
                        _ => unreachable!(),
                    }
                }
            }
            EventKind::PlayCard(card) => {
                hand.played |= card;
                hand.trick.play(card);
                if let Some(winning_card) = hand.trick.winner() {
                    let winner = hand.owner(winning_card);
                    hand.won[winner.idx()] |= hand.trick.cards;
                    hand.trick = Trick::new(winner);
                }
            }
        }
    }

    pub fn pass_cards(&mut self, player: Player, cards: Cards) -> Result<(), CardsError> {
        Ok(())
    }

    pub fn charge_cards(&mut self, player: Player, cards: Cards) -> Result<(), CardsError> {
        Ok(())
    }

    pub fn play_card(&mut self, player: Player, card: Card) -> Result<(), CardsError> {
        Ok(())
    }

    fn handle(&mut self, player: Player, event: RequestEvent) -> Result<(), CardsError> {
        let seat = if player == self.players[0] {
            Seat::North
        } else if player == self.players[1] {
            Seat::East
        } else if player == self.players[2] {
            Seat::South
        } else if player == self.players[3] {
            Seat::West
        } else {
            return Err(CardsError::IllegalPlayer(player));
        };
        for hand in &mut self.hands {
            if hand.state != HandState::Complete {
                return hand.handle(player, event, self.rules);
            }
        }
        Err(CardsError::GameComplete(self.id))
    }
}

impl Hand {
    fn new(pass_direction: PassDirection) -> Self {
        Self {
            state: if pass_direction == PassDirection::Keeper {
                HandState::KeeperCharging
            } else {
                HandState::Passing
            },
            pre_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            received_pass: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            done_charging: [false, false, false, false],
            charges: Cards::NONE,
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

    fn handle(
        &mut self,
        player: Player,
        event: RequestEvent,
        rules: ChargingRules,
    ) -> Result<(), CardsError> {
        match (self.state, event) {
            (HandState::KeeperCharging, RequestEvent::Charge(cards)) => {
                self.handle_charge(player, cards, rules)
            }
            (HandState::Passing, RequestEvent::Pass(cards)) => self.handle_pass(player, cards),
            (HandState::Charging, RequestEvent::Charge(cards)) => {
                self.handle_charge(player, cards, rules)
            }
            (HandState::Playing, RequestEvent::Play(card)) => self.handle_play(player, card),
            _ => Err(CardsError::IllegalAction(self.state)),
        }
    }

    fn handle_charge(
        &mut self,
        player: Player,
        cards: Cards,
        rules: ChargingRules,
    ) -> Result<(), CardsError> {
        Ok(())
    }

    fn handle_pass(&mut self, player: Player, cards: Cards) -> Result<(), CardsError> {
        Ok(())
    }

    fn handle_play(&mut self, player: Player, card: Card) -> Result<(), CardsError> {
        Ok(())
    }
}

enum RequestEvent {
    Pass(Cards),
    Charge(Cards),
    Play(Card),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub hand: PassDirection,
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
    ReceiveHand(Cards),
    ReceivePass(Cards),
    BlindCharge(u64),
    ChargeCards(Cards),
    PlayCard(Card),
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
    leader: Seat,
    lead: Option<Suit>,
    cards: Cards,
}

impl Trick {
    fn new(leader: Seat) -> Self {
        Self {
            leader,
            lead: None,
            cards: Cards::NONE,
        }
    }

    fn play(&mut self, card: Card) {
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
