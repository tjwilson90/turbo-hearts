import { LobbyPlayer, GameResult } from "../types";

export interface LobbyGame {
    gameId: string;
    players: LobbyPlayer[];
    createdBy: string;
    updatedAt: Date;
    createdAt: Date;
    startedAt: Date | undefined;
}

export interface IGameLinkSubstitutions {
    type: "game";
    gameId: string;
}
export interface IUserNameSubstitutions {
    type: "user";
    userId: string;
}
export type ISubstitutions = IGameLinkSubstitutions | IUserNameSubstitutions;

export interface ChatMessage {
    date: Date;
    userId: string | undefined;
    message: string;
    generated: boolean;
    substitutions: ISubstitutions[];
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
    lobby: ChatState;
}

export interface GamesState {
    [gameId: string]: LobbyGame;
}

export interface UiState {
    hideOldGames: boolean;
}

export interface LeaguesState {
    games: GameResult[];
}

export interface LobbyState {
    chats: ChatsState;
    games: GamesState;
    users: UsersState;
    leagues: LeaguesState;
    ui: UiState;
}
