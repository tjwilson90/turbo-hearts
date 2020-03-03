import { TypedAction } from "redoodle";
import { User, ChatMessage } from "./types";
import { Action, TurboHearts } from "../game/stateSnapshot";
import { Seat } from "../types";

export const SetGameUsers = TypedAction.define("setGameUsers")<{
  bottomSeat: Seat;
  top: User;
  right: User;
  bottom: User;
  left: User;
}>();
export const UpdateActions = TypedAction.define("updateActions")<{
  top: Action;
  right: Action;
  bottom: Action;
  left: Action;
}>();
export const UpdateUsers = TypedAction.define("updateUsers")<{ [key: string]: User }>();
export const AppendChat = TypedAction.define("appendChat")<ChatMessage>();
export const AppendTrick = TypedAction.define("appendTrick")<TurboHearts.Trick>();
export const ResetTricks = TypedAction.defineWithoutPayload("resetTricks")();
