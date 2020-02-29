use crate::{
    error::CardsError,
    types::{ChargingRules, GameId, Participant, Player, UserId},
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
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

struct Inner {
    subscribers: HashMap<UserId, Vec<UnboundedSender<LobbyEvent>>>,
    games: HashMap<GameId, HashSet<Participant>>,
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
                .map(|(game_id, players)| {
                    (
                        *game_id,
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

    pub async fn new_game(&self, user_id: UserId, rules: ChargingRules) -> GameId {
        let game_id = GameId::new();
        let mut inner = self.inner.lock().await;
        let mut game = HashSet::new();
        game.insert(Participant {
            player: Player::Human { user_id },
            rules,
        });
        inner.games.insert(game_id, game);
        inner.broadcast(LobbyEvent::NewGame { game_id, user_id });
        game_id
    }

    pub async fn join_game(
        &self,
        game_id: GameId,
        player: Player,
        rules: ChargingRules,
    ) -> Result<HashSet<Participant>, CardsError> {
        let mut inner = self.inner.lock().await;
        if let Some(players) = inner.games.get_mut(&game_id) {
            if players.len() == 4 {
                return Err(CardsError::GameHasStarted(game_id));
            }
            players.insert(Participant {
                player: player.clone(),
                rules,
            });
            let players = players.clone();
            inner.broadcast(LobbyEvent::JoinGame { game_id, player });
            Ok(players)
        } else {
            Err(CardsError::UnknownGame(game_id))
        }
    }

    pub async fn leave_game(&self, game_id: GameId, user_id: UserId) {
        let mut inner = self.inner.lock().await;
        let games = &mut inner.games;
        if let Entry::Occupied(mut entry) = games.entry(game_id) {
            let participants = entry.get_mut();
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
