import { combineReducers, TypedReducer } from "redoodle";
import { ChatsState, GamesState, UiState, UsersState } from "./types";
import {
    AppendChat, DeleteLobbyGame,
    ToggleCollapseGame,
    ToggleHideOldGames,
    UpdateChatUserIds,
    UpdateLobbyGame,
    UpdateUserNames
} from "./actions";

const usersReducer = TypedReducer.builder<UsersState>()
    .withHandler(UpdateUserNames.TYPE, (state, userNamesByUserId) => ({
        ...state,
        userNamesByUserId
    }))
    .build();

const gamesReducer = TypedReducer.builder<GamesState>()
    .withHandler(UpdateLobbyGame.TYPE, (state, { gameId, lobbyGame }) => ({
        ...state,
        [gameId]: lobbyGame,
    }))
    .withHandler(DeleteLobbyGame.TYPE, (state, { gameId}) => {
        const newState = { ...state };
        delete newState[gameId];
        return newState;
    })
    .build();

const chatsReducer = TypedReducer.builder<ChatsState>()
    .withHandler(AppendChat.TYPE, (state, { room, message }) => ({
        ...state,
        [room]: {
            ...state[room],
            messages: state[room].messages.concat([message]),
        }
    }))
    .withHandler(UpdateChatUserIds.TYPE, (state, { room, userIds }) => ({
        ...state,
        [room]: {
            ...state[room],
            userIds,
        }
    }))
    .build();

const uiReducer = TypedReducer.builder<UiState>()
    .withHandler(ToggleHideOldGames.TYPE, (state) => ({
        ...state,
        hideOldGames: !state.hideOldGames,
    }))
    .withHandler(ToggleCollapseGame.TYPE, (state, { gameId }) => ({
        ...state,
        collapsedGames: {
            ...state.collapsedGames,
            [gameId]: !state.collapsedGames[gameId],
        },
    }))
    .build();

export const rootReducer = combineReducers({
    chats: chatsReducer,
    users: usersReducer,
    games: gamesReducer,
    ui: uiReducer,
})
