use crate::{
    cards::{GameId, Player},
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
    games: HashMap<GameId, HashSet<Player>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Ping,
    EnterLobby(Player),
    CreateGame {
        id: GameId,
        player: Player,
    },
    LobbyState {
        viewers: Vec<Player>,
        games: HashMap<GameId, HashSet<Player>>,
    },
    JoinGame {
        id: GameId,
        player: Player,
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

    pub async fn enter(&self, player: Player, tx: UnboundedSender<Event>) {
        let mut inner = self.inner.lock().await;
        if inner.subscribers.remove(&player).is_none() {
            inner.broadcast(Event::EnterLobby(player.clone()));
        }
        tx.send(Event::LobbyState {
            viewers: inner.subscribers.keys().cloned().collect(),
            games: inner.games.clone(),
        })
        .unwrap();
        inner.subscribers.insert(player, tx);
    }

    pub async fn create_game(&self, id: GameId, player: Player) {
        let mut inner = self.inner.lock().await;
        inner
            .games
            .insert(id, HashSet::from_iter(vec![player.clone()]));
        inner.broadcast(Event::CreateGame { id, player });
    }

    pub async fn join_game(&self, id: GameId, player: Player) -> Result<Vec<Player>, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(players) = inner.games.get_mut(&id) {
            if players.insert(player.clone()) {
                let players = players.iter().cloned().collect();
                inner.broadcast(Event::JoinGame { id, player });
                Ok(players)
            } else {
                Ok(players.iter().cloned().collect())
            }
        } else {
            Err(CardsError::UnknownGame(id))
        }
    }

    pub async fn leave_game(&self, id: GameId, player: Player) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(id) {
            if entry.get_mut().remove(&player) {
                if entry.get().is_empty() {
                    entry.remove();
                }
                inner.broadcast(Event::LeaveGame { id, player });
            }
        }
    }

    pub async fn leave_lobby(&self, player: Player) {
        let mut inner = self.inner.lock().await;
        if inner.subscribers.remove(&player).is_some() {
            inner.broadcast(Event::LeaveLobby(player));
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
