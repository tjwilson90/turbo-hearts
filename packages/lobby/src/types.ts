export type BotStrategy = "random" | "duck" | "gotta_try" | "heuristic" | "simulate" | "neural_net";

export type Rules = "classic" | "blind" | "bridge" | "blind-bridge" | "chain" | "blind-chain";

export interface HumanLobbyPlayer {
    type: "human";
    userId: string;
}
export interface BotLobbyPlayer {
    type: "bot";
    userId: string;
    strategy: BotStrategy;
}
export type LobbyPlayer = HumanLobbyPlayer | BotLobbyPlayer;

export interface GameResult {
    gameId: string;
    time: number;
    players: {
        userId: string;
    }[];
    hands: {
        charges: string[];
        hearts: number[];
        qsWinnerId: string;
        tcWinnerId: string;
        jdWinnerId: string;
    }[];
}

export interface LobbyStateEvent {
    type: "lobby_state";
    subscribers: string[];
    games: {
        [gameId: string]: {
            updatedAt: Date;
            createdAt: Date;
            players: LobbyPlayer[];
            createdBy: string;
            startedAt: Date | undefined;
        };
    };
    chat: {
        timestamp: Date;
        userId: string;
        message: string;
    }[];
    userId: string;
    gameId: string;
}

export interface EnterLobbyEvent {
    type: "enter";
    userId: string;
}

export interface ExitLobbyEvent {
    type: "exit";
    userId: string;
}

export interface NewGameEvent {
    type: "new_game";
    createdBy: string;
    gameId: string;
}

export interface JoinGameEvent {
    type: "join_game";
    player: LobbyPlayer;
    gameId: string;
}

export interface LeaveGameEvent {
    type: "leave_game";
    player: LobbyPlayer;
    gameId: string;
}

export interface FinishGameEvent {
    type: "finish_game";
    gameId: string;
}

export interface StartGameEvent {
    type: "start_game";
    gameId: string;
    north: LobbyPlayer;
    east: LobbyPlayer;
    south: LobbyPlayer;
    west: LobbyPlayer;
}

export interface ChatEvent {
    type: "chat";
    userId: string;
    message: string;
}

export type LobbyEvent =
    | EnterLobbyEvent
    | ExitLobbyEvent
    | LobbyStateEvent
    | NewGameEvent
    | JoinGameEvent
    | LeaveGameEvent
    | FinishGameEvent
    | StartGameEvent
    | ChatEvent;
