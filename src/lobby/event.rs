use crate::types::{Event, GameId, Player, UserId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LobbyEvent {
    Ping,
    JoinLobby {
        user_id: UserId,
    },
    NewGame {
        game_id: GameId,
        user_id: UserId,
    },
    LobbyState {
        subscribers: HashSet<UserId>,
        games: HashMap<GameId, Vec<Player>>,
    },
    JoinGame {
        game_id: GameId,
        player: Player,
    },
    LeaveGame {
        game_id: GameId,
        user_id: UserId,
    },
    FinishGame {
        game_id: GameId,
    },
    Chat {
        user_id: UserId,
        message: String,
    },
    LeaveLobby {
        user_id: UserId,
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