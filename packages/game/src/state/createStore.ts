import * as cookie from "cookie";
import { combineReducers, createStore, loggingMiddleware, StoreEnhancer, TypedReducer } from "redoodle";
import { applyMiddleware, Store } from "redux";
import { Snapshotter } from "../game/snapshotter";
import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import {
  AppendChat,
  SetGameUsers,
  UpdateUsers,
  UpdateActions,
  AppendTrick,
  ResetTricks,
  AppendHandScore,
  EnableSpectatorMode,
  SetLocalPass,
  UpdateClaims,
  ResetClaims
} from "./actions";
import { ChatState, GameAppState, GameContext, GameState, User, UsersState } from "./types";
import { TrickTracker } from "../game/TrickTracker";
import { PassTracker } from "../game/PassTracker";

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
  .withHandler(EnableSpectatorMode.TYPE, state => {
    return {
      ...state,
      spectatorMode: true
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
  .withHandler(SetLocalPass.TYPE, (state, localPass) => {
    return {
      ...state,
      localPass
    };
  })
  .withHandler(UpdateClaims.TYPE, (state, claimUpdate) => {
    switch (claimUpdate.type) {
      case "claim":
        return {
          ...state,
          claims: {
            ...state.claims,
            [claimUpdate.seat]: {
              [claimUpdate.seat]: true,
            },
          }
        };
      case "accept_claim":
        return {
          ...state,
          claims: {
            ...state.claims,
            [claimUpdate.claimer]: {
              ...state.claims[claimUpdate.claimer],
              [claimUpdate.acceptor]: "ACCEPT",
            },
          }
        };
      case "reject_claim":
        return {
          ...state,
          claims: {
            ...state.claims,
            [claimUpdate.claimer]: {
              ...state.claims[claimUpdate.claimer],
              [claimUpdate.rejector]: "REJECT",
            },
          }
        };
      }
  })
  .withHandler(ResetClaims.TYPE, (state) => {
    return {
      ...state,
      claims: {},
    }
  })
  .withHandler(AppendHandScore.TYPE, (state, handScores) => {
    return {
      ...state,
      scores: [
        ...state.scores,
        [handScores.northScore, handScores.eastScore, handScores.southScore, handScores.westScore]
      ]
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
    spectatorMode: false,
    bottomSeat: undefined!,
    top: undefined,
    right: undefined,
    bottom: undefined,
    left: undefined,
    topAction: "none",
    rightAction: "none",
    bottomAction: "none",
    leftAction: "none",
    scores: [],
    tricks: [],
    localPass: undefined,
    claims: {},
  },
  context: {
    eventSource: undefined!,
    service: undefined!,
    snapshotter: undefined!,
    trickTracker: undefined!,
    passTracker: undefined!
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
  initialState.context.passTracker = new PassTracker(initialState.users.me.userId);

  initialState.context.eventSource.on("event", event => {
    initialState.context.snapshotter.onEvent(event);
    initialState.context.trickTracker.onEvent(event);
    initialState.context.passTracker.onEvent(event);
  });
  return createStore(
    rootReducer,
    initialState,
    (applyMiddleware(loggingMiddleware({})) as any) as StoreEnhancer
  ) as Store<GameAppState>;
}
