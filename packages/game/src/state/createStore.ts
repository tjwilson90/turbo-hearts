import * as cookie from "cookie";
import { GameAppState, ChatState, UsersState, GameState, GameContext } from "./types";
import { createStore, TypedReducer, combineReducers } from "redoodle";
import { Store } from "redux";
import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { Snapshotter } from "../game/snapshotter";
import { SetUsers } from "./actions";

const chatReducer = TypedReducer.builder<ChatState>().build();
const usersReducer = TypedReducer.builder<UsersState>().build();
const gameReducer = TypedReducer.builder<GameState>()
  .withHandler(SetUsers.TYPE, (state, users) => {
    return {
      ...state,
      ...users
    };
  })
  .build();

const rootReducer = combineReducers({
  chat: chatReducer,
  users: usersReducer,
  game: gameReducer,
  context: TypedReducer.builder<GameContext>().build()
});

const INITIAL_STATE: GameAppState = {
  chat: {
    messages: []
  },
  users: {
    users: {},
    me: undefined!
  },
  game: {
    gameId: undefined!,
    top: undefined,
    right: undefined,
    bottom: undefined,
    left: undefined
  },
  context: {
    eventSource: undefined!,
    service: undefined!,
    snapshotter: undefined!
  }
};

export function createGameAppStore(gameId: string) {
  const cookieParams = cookie.parse(document.cookie);
  const initialState = INITIAL_STATE;
  initialState.users.me = {
    userId: cookieParams["USER_ID"],
    name: cookieParams["USER_NAME"]
  };
  initialState.game.gameId = gameId;
  initialState.context.eventSource = new TurboHeartsEventSource(gameId);
  initialState.context.service = new TurboHeartsService(gameId);
  initialState.context.snapshotter = new Snapshotter(initialState.users.me.userId);
  initialState.context.eventSource.on("event", initialState.context.snapshotter.onEvent);
  return createStore(rootReducer, initialState) as Store<GameAppState>;
}
