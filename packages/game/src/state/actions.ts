import { TypedAction } from "redoodle";
import { User, ChatMessage } from "./types";
import { Action } from "../game/stateSnapshot";

export const SetGameUsers = TypedAction.define("setGameUsers")<{ top: User; right: User; bottom: User; left: User }>();
export const UpdateActions = TypedAction.define("updateActions")<{
  top: Action;
  right: Action;
  bottom: Action;
  left: Action;
}>();
export const UpdateUsers = TypedAction.define("updateUsers")<{ [key: string]: User }>();
export const AppendChat = TypedAction.define("appendChat")<ChatMessage>();
