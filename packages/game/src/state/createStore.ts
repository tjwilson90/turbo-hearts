import * as cookie from "cookie";
import { combineReducers, createStore, loggingMiddleware, StoreEnhancer, TypedReducer } from "redoodle";
import { applyMiddleware, Store } from "redux";
import { Snapshotter } from "../game/snapshotter";
import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { AppendChat, SetGameUsers, UpdateUsers, UpdateActions, AppendTrick, ResetTricks } from "./actions";
import { ChatState, GameAppState, GameContext, GameState, User, UsersState } from "./types";
import { TrickTracker } from "../game/TrickTracker";

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
  .withHandler(UpdateActions.TYPE, (state, actions) => {
    return {
      ...state,
      topAction: actions.top,
      rightAction: actions.right,
      bottomAction: actions.bottom,
      leftAction: actions.left
    };
  })
  .withHandler(AppendTrick.TYPE, (state, trick) => {
    return {
      ...state,
      tricks: [...state.tricks, trick]
    };
  })
  .withHandler(ResetTricks.TYPE, state => {
    return {
      ...state,
      tricks: []
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
    left: undefined,
    topAction: "none",
    rightAction: "none",
    bottomAction: "none",
    leftAction: "none",
    tricks: []
  },
  context: {
    eventSource: undefined!,
    service: undefined!,
    snapshotter: undefined!,
    trickTracker: undefined!
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
  initialState.context.trickTracker = new TrickTracker();

  initialState.context.eventSource.on("event", event => {
    initialState.context.snapshotter.onEvent(event);
    initialState.context.trickTracker.onEvent(event);
  });
  return createStore(
    rootReducer,
    initialState,
    (applyMiddleware(loggingMiddleware({})) as any) as StoreEnhancer
  ) as Store<GameAppState>;
}
