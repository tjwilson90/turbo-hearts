import { TypedAction } from "redoodle";
import { User, ChatMessage } from "./types";

export const SetGameUsers = TypedAction.define("setGameUsers")<{ top: User; right: User; bottom: User; left: User }>();
export const UpdateUsers = TypedAction.define("updateUsers")<{ [key: string]: User }>();
export const AppendChat = TypedAction.define("appendChat")<ChatMessage>();
