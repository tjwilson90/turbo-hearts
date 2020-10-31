use crate::{BotStrategy, ChargingRules, GameId, Player, PlayerWithOptions, Seat, Seed, UserId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewGameRequest {
    pub rules: ChargingRules,
    pub seat: Option<Seat>,
    pub seed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinGameRequest {
    pub game_id: GameId,
    pub rules: ChargingRules,
    pub seat: Option<Seat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartGameRequest {
    pub game_id: GameId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveGameRequest {
    pub game_id: GameId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddBotRequest {
    pub game_id: GameId,
    pub rules: ChargingRules,
    pub strategy: BotStrategy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePlayerRequest {
    pub game_id: GameId,
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LobbyChatRequest {
    pub message: String,
}

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
