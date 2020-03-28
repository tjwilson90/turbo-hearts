import { TypedAction } from "redoodle";
import { ChatMessage, LobbyGame } from "./types";
import { GameResult } from "../types";

export const UpdateUserNames = TypedAction.define("UpdateUserNames")<{ [userId: string]: string }>();

export const AppendChat = TypedAction.define("AppendChatMessage")<{
    room: "lobby";
    message: ChatMessage;
}>();
export const UpdateChatUserIds = TypedAction.define("UpdateChatUserIds")<{
    room: "lobby";
    userIds: string[];
}>();

export const UpdateLobbyGame = TypedAction.define("UpdateLobbyGame")<{
    gameId: string;
    lobbyGame: LobbyGame;
}>();
export const DeleteLobbyGame = TypedAction.define("DeleteLobbyGame")<{
    gameId: string;
}>();

export const ToggleHideOldGames = TypedAction.defineWithoutPayload("ToggleHideOldGames")();

export const SetLeagueGames = TypedAction.define("SetLeagueGames")<GameResult[]>();
