use crate::lobby::{LobbyChat, LobbyGame};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use turbo_hearts_api::{GameId, Player, PlayerWithOptions, Seed, UserId};

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
