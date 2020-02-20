import * as cookie from "cookie";
import { PlaySubmitter } from "./game/PlaySubmitter";
import { TurboHearts } from "./game/TurboHearts";
import { TurboHeartsEventSource } from "./game/TurboHeartsEventSource";
import "./styles/style.scss";

document.addEventListener("DOMContentLoaded", () => {
  const userId = cookie.parse(document.cookie)["name"];
  if (userId?.length === 0) {
    document.body.innerHTML = "Missing user id";
    return;
  }
  const gameId = window.location.hash.substring(1);
  if (gameId.length !== 36) {
    document.body.innerHTML = "Missing game id";
    return;
  }
  const submitter = new PlaySubmitter(gameId);
  const th = new TurboHearts(document.getElementById("turbo-hearts") as HTMLCanvasElement, userId, submitter);
  new TurboHeartsEventSource(th, gameId);
});
