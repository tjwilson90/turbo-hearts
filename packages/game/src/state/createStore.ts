import * as cookie from "cookie";
import { GameAppState, ChatState, UsersState, GameState, GameContext, User } from "./types";
import { createStore, TypedReducer, combineReducers, loggingMiddleware, StoreEnhancer } from "redoodle";
import { Store, applyMiddleware } from "redux";
import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { Snapshotter } from "../game/snapshotter";
import { SetGameUsers, AppendChat, UpdateUsers } from "./actions";
import { ChatEvent } from "../types";

const chatReducer = TypedReducer.builder<ChatState>()
  .withHandler(AppendChat.TYPE, (state, msg) => {
    return {
      ...state,
      messages: [...state.messages, msg]
    };
  })
  .build();

const usersReducer = TypedReducer.builder<UsersState>()
  .withHandler(UpdateUsers.TYPE, (state, users) => {
    const toUpdate: User[] = [];
    for (const id in users) {
      if (state.users[id] !== users[id]) {
        toUpdate.push(users[id]);
      }
    }
    if (toUpdate.length === 0) {
      return state;
    } else {
      const newUsers = {
        ...state.users
      };
      for (const user of toUpdate) {
        newUsers[user.userId] = user;
      }
      return {
        ...state,
        users: newUsers
      };
    }
  })
  .build();

const gameReducer = TypedReducer.builder<GameState>()
  .withHandler(SetGameUsers.TYPE, (state, users) => {
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
  return createStore(rootReducer, initialState, applyMiddleware(loggingMiddleware({})) as StoreEnhancer) as Store<
    GameAppState
  >;
}
