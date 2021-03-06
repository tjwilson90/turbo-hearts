import { TurboHeartsEventSource } from "../game/TurboHeartsEventSource";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { Snapshotter } from "../game/snapshotter";
import { Action, TurboHearts } from "../game/stateSnapshot";
import { TrickTracker } from "../game/TrickTracker";
import { Seat } from "../types";
import { PassTracker } from "../game/PassTracker";

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

export type ClaimResponseStatus = "ACCEPT" | "REJECT";

export interface ClaimStatus {
  north?: ClaimResponseStatus;
  east?: ClaimResponseStatus;
  south?: ClaimResponseStatus;
  west?: ClaimResponseStatus;
}

export interface GameState {
  gameId: string;
  spectatorMode: boolean;
  bottomSeat: Seat;

  top: User | undefined;
  right: User | undefined;
  bottom: User | undefined;
  left: User | undefined;

  topAction: Action;
  rightAction: Action;
  bottomAction: Action;
  leftAction: Action;

  scores: number[][];
  tricks: TurboHearts.Trick[];
  localPass: TurboHearts.LocalPass | undefined;
  claims: { [key in Seat]?: ClaimStatus };
}

export interface GameContext {
  eventSource: TurboHeartsEventSource;
  service: TurboHeartsService;
  snapshotter: Snapshotter;
  trickTracker: TrickTracker;
  passTracker: PassTracker;
}

export interface GameAppState {
  chat: ChatState;
  users: UsersState;
  game: GameState;
  context: GameContext;
}
