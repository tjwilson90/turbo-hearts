import * as cookie from "cookie";
import { createStore as createRedoodleStore, loggingMiddleware, StoreEnhancer } from "redoodle";
import { applyMiddleware, Store } from "redux";
import { LobbyState } from "./types";
import { rootReducer } from "./reducers";

export function createStore(): Store<LobbyState> {
    const cookieParams = cookie.parse(document.cookie);

    const initialState: LobbyState = {
        chats: {
            lobby: {
                messages: [],
                userIds: [],
            }
        },
        games: {},
        users: {
            ownUserId: cookieParams["USER_ID"],
            userNamesByUserId: {
                [cookieParams["USER_ID"]]: cookieParams["USER_NAME"],
            },
        },
        ui: {
            collapsedGames: {},
            hideOldGames: true,
        }
    }

    return createRedoodleStore(
        rootReducer,
        initialState,
        (applyMiddleware(loggingMiddleware({})) as any) as StoreEnhancer
    ) as Store<LobbyState>
}

