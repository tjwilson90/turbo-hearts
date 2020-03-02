import { TypedAction } from "redoodle";
import { User } from "./types";

export const SetUsers = TypedAction.define("setUsers")<{ top: User; right: User; bottom: User; left: User }>();
