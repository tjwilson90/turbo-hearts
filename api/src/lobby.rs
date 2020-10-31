use crate::{GameId, Player, PlayerWithOptions, Seed, UserId};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[serde(tag = "type", rename_all = "snake_case")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum LobbyEvent {
    Ping,
    JoinLobby {
        user_id: UserId,
    },
    NewGame {
        game_id: GameId,
        player: PlayerWithOptions,
        seed: Seed,
    },
    LobbyState {
        subscribers: HashSet<UserId>,
        chat: Vec<LobbyChat>,
        games: HashMap<GameId, LobbyGame>,
    },
    JoinGame {
        game_id: GameId,
        player: PlayerWithOptions,
    },
    StartGame {
        game_id: GameId,
        north: Player,
        east: Player,
        south: Player,
        west: Player,
    },
    LeaveGame {
        game_id: GameId,
        player: Player,
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

impl LobbyEvent {
    pub fn is_ping(&self) -> bool {
        match self {
            LobbyEvent::Ping => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LobbyGame {
    pub players: HashSet<PlayerWithOptions>,
    pub seed: Seed,
    pub created_time: i64,
    pub created_by: UserId,
    pub last_updated_time: i64,
    pub last_updated_by: UserId,
    pub started_time: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LobbyChat {
    pub timestamp: i64,
    pub user_id: UserId,
    pub message: String,
}
