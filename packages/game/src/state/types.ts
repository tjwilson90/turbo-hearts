import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { Snapshotter } from "../game/snapshotter";
import { Action } from "../game/stateSnapshot";

export interface ChatMessage {
  userId: string;
  message: string;
}

export interface ChatState {
  messages: ChatMessage[];
}

export interface User {
  userId: string;
  name: string;
}

export interface UsersState {
  users: { [key: string]: User };
  me: User;
}

export interface GameState {
  gameId: string;
  top: User | undefined;
  right: User | undefined;
  bottom: User | undefined;
  left: User | undefined;

  topAction: Action;
  rightAction: Action;
  bottomAction: Action;
  leftAction: Action;
}

export interface GameContext {
  eventSource: TurboHeartsEventSource;
  service: TurboHeartsService;
  snapshotter: Snapshotter;
}

export interface GameAppState {
  chat: ChatState;
  users: UsersState;
  game: GameState;
  context: GameContext;
}
