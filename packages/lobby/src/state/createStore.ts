import * as cookie from "cookie";
import { createStore as createRedoodleStore, loggingMiddleware, StoreEnhancer } from "redoodle";
import { applyMiddleware, Store } from "redux";
import { LobbyState } from "./types";
import { rootReducer } from "./reducers";

export function createStore(): Store<LobbyState> {
    const cookieParams = cookie.parse(document.cookie);
    const userId = cookieParams["USER_ID"];

    const initialState: LobbyState = {
        chats: {
            lobby: {
                messages: [{
                    date: new Date(),
                    userId,
                    message: "joined.",
                    generated: true,
                }],
                userIds: [],
            }
        },
        games: {},
        users: {
            ownUserId: userId,
            userNamesByUserId: {
                [userId]: cookieParams["USER_NAME"],
            },
        },
        ui: {
            hideOldGames: true,
        }
    }

    return createRedoodleStore(
        rootReducer,
        initialState,
        (applyMiddleware(loggingMiddleware({})) as any) as StoreEnhancer
    ) as Store<LobbyState>
}

