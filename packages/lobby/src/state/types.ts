import { LobbyPlayer } from "../types";

export interface LobbyGame {
    gameId: string;
    players: LobbyPlayer[];
    createdBy: string;
    updatedAt: Date;
    createdAt: Date;
}

export interface ChatMessage {
    date: Date;
    userId: string | undefined;
    message: string;
    generated: boolean;
}

export interface UsersState {
    userNamesByUserId: { [key: string]: string };
    ownUserId: string;
}

export interface ChatState {
    userIds: string[];
    messages: ChatMessage[];
}

export interface ChatsState {
    lobby: ChatState
}

export interface GamesState {
    [gameId: string]: LobbyGame;
}

export interface UiState {
    hideOldGames: boolean;
}

export interface LobbyState {
    chats: ChatsState;
    games: GamesState;
    users: UsersState;
    ui: UiState;
}
