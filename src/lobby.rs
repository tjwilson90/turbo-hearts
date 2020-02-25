use crate::{
    error::CardsError,
    types::{ChargingRules, Event, GameId, Participant, Player},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Clone)]
pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    subscribers: HashMap<String, Vec<UnboundedSender<LobbyEvent>>>,
    games: HashMap<GameId, HashSet<Participant>>,
}

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LobbyEvent {
    Ping,
    JoinLobby {
        name: String,
    },
    NewGame {
        id: GameId,
        name: String,
    },
    LobbyState {
        subscribers: HashSet<String>,
        games: HashMap<GameId, Vec<Player>>,
    },
    JoinGame {
        id: GameId,
        player: Player,
    },
    LeaveGame {
        id: GameId,
        name: String,
    },
    FinishGame {
        id: GameId,
    },
    Chat {
        name: String,
        message: String,
    },
    LeaveLobby {
        name: String,
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
    pub fn new(games: HashMap<GameId, HashSet<Participant>>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                subscribers: HashMap::new(),
                games,
            })),
        }
    }

    pub async fn ping(&self) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Ping);
    }

    pub async fn subscribe(&self, name: String) -> UnboundedReceiver<LobbyEvent> {
        let (tx, rx) = unbounded_channel();
        let mut inner = self.inner.lock().await;
        if !inner.subscribers.contains_key(&name) {
            inner.broadcast(LobbyEvent::JoinLobby { name: name.clone() });
        }
        inner
            .subscribers
            .entry(name)
            .or_insert(Vec::new())
            .push(tx.clone());
        tx.send(LobbyEvent::LobbyState {
            subscribers: inner.subscribers.keys().cloned().collect(),
            games: inner
                .games
                .iter()
                .map(|(id, players)| {
                    (
                        *id,
                        players
                            .into_iter()
                            .map(|participant| &participant.player)
                            .cloned()
                            .collect(),
                    )
                })
                .collect(),
        })
        .unwrap();
        rx
    }

    pub async fn new_game(&self, name: String, rules: ChargingRules) -> GameId {
        let id = GameId::new();
        let mut inner = self.inner.lock().await;
        let mut game = HashSet::new();
        game.insert(Participant {
            player: Player::Human { name: name.clone() },
            rules,
        });
        inner.games.insert(id, game);
        inner.broadcast(LobbyEvent::NewGame { id, name });
        id
    }

    pub async fn join_game(
        &self,
        id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<HashSet<Participant>, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(players) = inner.games.get_mut(&id) {
            if players.len() == 4 {
                return Err(CardsError::GameHasStarted(id));
            }
            players.insert(Participant {
                player: player.clone(),
                rules,
            });
            let players = players.clone();
            inner.broadcast(LobbyEvent::JoinGame { id, player });
            Ok(players)
        } else {
            Err(CardsError::UnknownGame(id))
        }
    }

    pub async fn leave_game(&self, id: GameId, name: String) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(id) {
            let participants = entry.get_mut();
            let count = participants.len();
            participants.retain(|participant| participant.player.name() != &name);
            if participants.len() < count {
                if participants.is_empty() {
                    entry.remove();
                }
                inner.broadcast(LobbyEvent::LeaveGame { id, name });
            }
        }
    }

    pub async fn remove_game(&self, id: GameId) {
        let mut inner = self.inner.lock().await;
        if inner.games.remove(&id).is_some() {
            inner.broadcast(LobbyEvent::FinishGame { id });
        }
    }

    pub async fn chat(&self, name: String, message: String) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Chat { name, message });
    }
}

impl Inner {
    fn broadcast(&mut self, event: LobbyEvent) {
        let mut events = VecDeque::new();
        events.push_back(event);
        while let Some(event) = events.pop_front() {
            self.subscribers.retain(|name, txs| {
                txs.retain(|tx| tx.send(event.clone()).is_ok());
                if txs.is_empty() {
                    events.push_back(LobbyEvent::LeaveLobby { name: name.clone() });
                    false
                } else {
                    true
                }
            });
        }
    }
}
