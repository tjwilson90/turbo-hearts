use crate::{
    error::CardsError,
    hacks::{unbounded_channel, Mutex, UnboundedReceiver, UnboundedSender},
    types::{ChargingRules, Event, GameId, Player},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
};

#[derive(Clone)]
pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    subscribers: HashMap<Player, UnboundedSender<LobbyEvent>>,
    games: HashMap<GameId, HashMap<Player, ChargingRules>>,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LobbyEvent {
    Ping,
    Subscribe {
        player: Player,
    },
    NewGame {
        id: GameId,
        player: Player,
        rules: ChargingRules,
    },
    LobbyState {
        subscribers: HashSet<Player>,
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
    LeaveLobby {
        player: Player,
    },
}

impl Event for LobbyEvent {
    fn is_ping(&self) -> bool {
        match self {
            LobbyEvent::Ping => true,
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
        inner.broadcast(LobbyEvent::Ping);
    }

    pub async fn subscribe(&self, player: Player) -> UnboundedReceiver<LobbyEvent> {
        let (tx, rx) = unbounded_channel();
        let mut inner = self.inner.lock().await;
        if inner.subscribers.remove(&player).is_none() {
            inner.broadcast(LobbyEvent::Subscribe {
                player: player.clone(),
            });
        }
        inner.subscribers.insert(player, tx.clone());
        tx.send(LobbyEvent::LobbyState {
            subscribers: inner.subscribers.keys().cloned().collect(),
            games: inner.games.clone(),
        })
        .unwrap();
        rx
    }

    pub async fn new_game(&self, player: Player, rules: ChargingRules) -> GameId {
        let id = GameId::new();
        let mut inner = self.inner.lock().await;
        let mut game = HashMap::new();
        game.insert(player.clone(), rules);
        inner.games.insert(id, game);
        inner.broadcast(LobbyEvent::NewGame { id, player, rules });
        id
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
            inner.broadcast(LobbyEvent::JoinGame { id, player, rules });
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
                inner.broadcast(LobbyEvent::LeaveGame { id, player });
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

    fn broadcast(&mut self, event: LobbyEvent) {
        let mut events = VecDeque::new();
        events.push_back(event);
        while let Some(event) = events.pop_front() {
            self.subscribers.retain(|p, tx| {
                if tx.send(event.clone()).is_ok() {
                    true
                } else {
                    events.push_back(LobbyEvent::LeaveLobby { player: p.clone() });
                    false
                }
            });
        }
    }
}
