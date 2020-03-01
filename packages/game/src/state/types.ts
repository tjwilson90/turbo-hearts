import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { Snapshotter } from "../game/snapshotter";

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
