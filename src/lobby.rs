use crate::{
    cards::{ChargingRules, GameId, Player},
    error::CardsError,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    iter::FromIterator,
    sync::Arc,
};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    subscribers: HashMap<Player, UnboundedSender<Event>>,
    games: HashMap<GameId, HashMap<Player, ChargingRules>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Ping,
    Subscribe(Player),
    NewGame {
        id: GameId,
        player: Player,
        rules: ChargingRules,
    },
    LobbyState {
        viewers: Vec<Player>,
        games: HashMap<GameId, HashMap<Player, ChargingRules>>,
    },
    JoinGame {
        id: GameId,
        player: Player,
        rules: ChargingRules,
    },
    LeaveGame {
        id: GameId,
        player: Player,
    },
    LeaveLobby(Player),
}

impl Event {
    pub fn is_ping(&self) -> bool {
        match self {
            Event::Ping => true,
            _ => false,
        }
    }
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(Event::Ping);
    }

    pub async fn subscribe(&self, player: Player, tx: UnboundedSender<Event>) {
        let mut inner = self.inner.lock().await;
        if inner.subscribers.remove(&player).is_none() {
            inner.broadcast(Event::Subscribe(player.clone()));
        }
        tx.send(Event::LobbyState {
            viewers: inner.subscribers.keys().cloned().collect(),
            games: inner.games.clone(),
        })
        .unwrap();
        inner.subscribers.insert(player, tx);
    }

    pub async fn new_game(&self, id: GameId, player: Player, rules: ChargingRules) {
        let mut inner = self.inner.lock().await;
        let mut game = HashMap::new();
        game.insert(player.clone(), rules);
        inner.games.insert(id, game);
        inner.broadcast(Event::NewGame { id, player, rules });
    }

    pub async fn join_game(
        &self,
        id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<HashMap<Player, ChargingRules>, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(players) = inner.games.get_mut(&id) {
            players.insert(player.clone(), rules);
            let players = players.clone();
            inner.broadcast(Event::JoinGame { id, player, rules });
            Ok(players)
        } else {
            Err(CardsError::UnknownGame(id))
        }
    }

    pub async fn leave_game(&self, id: GameId, player: Player) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(id) {
            if entry.get_mut().remove(&player).is_some() {
                if entry.get().is_empty() {
                    entry.remove();
                }
                inner.broadcast(Event::LeaveGame { id, player });
            }
        }
    }
}

impl Inner {
    fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            games: HashMap::new(),
        }
    }

    fn broadcast(&mut self, event: Event) {
        let mut events = VecDeque::new();
        events.push_back(event);
        while let Some(event) = events.pop_front() {
            self.subscribers.retain(|p, tx| {
                if tx.send(event.clone()).is_ok() {
                    true
                } else {
                    events.push_back(Event::LeaveLobby(p.clone()));
                    false
                }
            });
        }
    }
}
