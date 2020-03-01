use crate::{
    error::CardsError,
    types::{ChargingRules, GameId, Participant, Player, UserId},
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::SystemTime,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    Mutex,
};

mod endpoints;
mod event;

pub use endpoints::*;
pub use event::*;

#[derive(Clone)]
pub struct Lobby {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Clone)]
pub struct GameLobby {
    pub participants: HashSet<Participant>,
    pub created_at: i64,
    pub updated_at: i64,
}

struct Inner {
    subscribers: HashMap<UserId, Vec<UnboundedSender<LobbyEvent>>>,
    games: HashMap<GameId, GameLobby>,
}

impl Lobby {
    pub fn new(games: HashMap<GameId, GameLobby>) -> Self {
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

    pub async fn subscribe(&self, user_id: UserId) -> UnboundedReceiver<LobbyEvent> {
        let (tx, rx) = unbounded_channel();
        let mut inner = self.inner.lock().await;
        if !inner.subscribers.contains_key(&user_id) {
            inner.broadcast(LobbyEvent::JoinLobby { user_id });
        }
        inner
            .subscribers
            .entry(user_id)
            .or_insert(Vec::new())
            .push(tx.clone());
        tx.send(LobbyEvent::LobbyState {
            subscribers: inner.subscribers.keys().cloned().collect(),
            games: inner
                .games
                .iter()
                .map(|(game_id, lobby)| {
                    (
                        *game_id,
                        lobby
                            .participants
                            .clone()
                            .into_iter()
                            .map(|participant| participant.player.clone())
                            .collect(),
                    )
                })
                .collect(),
        })
        .unwrap();
        rx
    }

    pub async fn new_game(&self, user_id: UserId, rules: ChargingRules) -> GameId {
        let game_id = GameId::new();
        let mut inner = self.inner.lock().await;
        let mut participants = HashSet::new();
        participants.insert(Participant {
            player: Player::Human { user_id },
            rules,
        });
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let game = GameLobby {
            participants,
            created_at: timestamp,
            updated_at: timestamp,
        };
        inner.games.insert(game_id, game);
        inner.broadcast(LobbyEvent::NewGame { game_id, user_id });
        game_id
    }

    pub async fn join_game(
        &self,
        game_id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<GameLobby, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(lobby) = inner.games.get_mut(&game_id) {
            if lobby.participants.len() == 4 {
                return Err(CardsError::GameHasStarted(game_id));
            }
            lobby.participants.insert(Participant {
                player: player.clone(),
                rules,
            });
            lobby.updated_at = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64;
            let lobby = lobby.clone();
            inner.broadcast(LobbyEvent::JoinGame { game_id, player });
            Ok(lobby)
        } else {
            Err(CardsError::UnknownGame(game_id))
        }
    }

    pub async fn leave_game(&self, game_id: GameId, user_id: UserId) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(game_id) {
            let participants = &mut entry.get_mut().participants;
            let count = participants.len();
            participants.retain(|participant| participant.player.user_id() != user_id);
            if participants.len() < count {
                if participants.is_empty() {
                    entry.remove();
                }
                inner.broadcast(LobbyEvent::LeaveGame { game_id, user_id });
            }
        }
    }

    pub async fn remove_game(&self, game_id: GameId) {
        let mut inner = self.inner.lock().await;
        if inner.games.remove(&game_id).is_some() {
            inner.broadcast(LobbyEvent::FinishGame { game_id });
        }
    }

    pub async fn chat(&self, user_id: UserId, message: String) {
        let mut inner = self.inner.lock().await;
        inner.broadcast(LobbyEvent::Chat { user_id, message });
    }
}

impl Inner {
    fn broadcast(&mut self, event: LobbyEvent) {
        let mut events = VecDeque::new();
        events.push_back(event);
        while let Some(event) = events.pop_front() {
            self.subscribers.retain(|user_id, txs| {
                txs.retain(|tx| tx.send(event.clone()).is_ok());
                if txs.is_empty() {
                    events.push_back(LobbyEvent::LeaveLobby { user_id: *user_id });
                    false
                } else {
                    true
                }
            });
        }
    }
}
