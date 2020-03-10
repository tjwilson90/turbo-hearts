use crate::{
    game::id::GameId,
    lobby::{LobbyChat, LobbyGame},
    player::PlayerWithOptions,
    seed::Seed,
    user::UserId,
};
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
        north: UserId,
        east: UserId,
        south: UserId,
        west: UserId,
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

impl LobbyEvent {
    pub fn is_ping(&self) -> bool {
        match self {
            LobbyEvent::Ping => true,
            _ => false,
        }
    }
}
