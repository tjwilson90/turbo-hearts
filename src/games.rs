use crate::{
    cards::{Card, Cards, ChargingRules, EventId, GameId, Player, Seat},
    error::CardsError,
};
use rusqlite::{
    types::{FromSql, FromSqlError, ToSqlOutput, Value, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

#[derive(Clone)]
pub struct Games {
    inner: Arc<Mutex<Inner>>,
}

impl Games {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                subscribers: HashMap::new(),
                games: HashMap::new(),
            })),
        }
    }

    pub async fn subscribe(&self, id: GameId, player: Player, tx: UnboundedSender<Event>) {
        let mut inner = self.inner.lock().await;
        inner.subscribers.insert(player, tx);
    }

    pub async fn pass_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let mut inner = self.inner.lock().await;
        match inner.games.get_mut(&id) {
            Some(game) => game.pass_cards(player, cards),
            None => Err(CardsError::UnknownGame(id)),
        }
    }

    pub async fn charge_cards(
        &self,
        id: GameId,
        player: Player,
        cards: Cards,
    ) -> Result<(), CardsError> {
        let mut inner = self.inner.lock().await;
        match inner.games.get_mut(&id) {
            Some(game) => game.charge_cards(player, cards),
            None => Err(CardsError::UnknownGame(id)),
        }
    }

    pub async fn play_card(
        &self,
        id: GameId,
        player: Player,
        card: Card,
    ) -> Result<(), CardsError> {
        let mut inner = self.inner.lock().await;
        match inner.games.get_mut(&id) {
            Some(game) => game.play_card(player, card),
            None => Err(CardsError::UnknownGame(id)),
        }
    }
}

struct Inner {
    subscribers: HashMap<Player, UnboundedSender<Event>>,
    games: HashMap<GameId, Game>,
}

struct Game {
    id: GameId,
    rules: ChargingRules,
    subscribers: HashSet<Player>,
    players: [Player; 4],
    hands: [Hand; 4],
}

struct Hand {
    state: HandState,
    pass_direction: PassDirection,
    pre_pass_hand: [Cards; 4],

    keeper_charge_sequence: Vec<(Seat, Cards)>,

    received_pass: [Cards; 4],
    post_pass_hand: [Cards; 4],

    charge_sequence: Vec<(Seat, Cards)>,
    final_charges: [Cards; 4],

    current_hand: [Cards; 4],
    trick_leader: Seat,
    current_trick: Vec<Card>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandState {
    KeeperCharging,
    Passing,
    Charging,
    Playing,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PassDirection {
    Left,
    Right,
    Across,
    Keeper,
}

impl Game {
    fn new(
        id: GameId,
        rules: ChargingRules,
        north: Player,
        east: Player,
        south: Player,
        west: Player,
    ) -> Self {
        Self {
            id,
            rules,
            subscribers: HashSet::new(),
            players: [north, east, south, west],
            hands: [
                Hand::new(PassDirection::Left),
                Hand::new(PassDirection::Right),
                Hand::new(PassDirection::Across),
                Hand::new(PassDirection::Keeper),
            ],
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
            pass_direction,
            pre_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            keeper_charge_sequence: Vec::new(),
            received_pass: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            post_pass_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            charge_sequence: Vec::new(),
            final_charges: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            current_hand: [Cards::NONE, Cards::NONE, Cards::NONE, Cards::NONE],
            trick_leader: Seat::North,
            current_trick: Vec::new(),
        }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_id: EventId,
    pub player: Player,
    pub timestamp: u64,
    pub event: EventKind,
}

impl Event {
    pub fn is_ping(&self) -> bool {
        match self.event {
            EventKind::Ping => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
