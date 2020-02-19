import { ChargeEvent } from "./events/ChargeEvent";
import { DealEvent } from "./events/DealEvent";
import { EndTrickEvent } from "./events/EndTrickEvent";
import { PlayEvent } from "./events/PlayEvent";
import { ReceivePassEvent } from "./events/ReceivePassEvent";
import { SendPassEvent } from "./events/SendPassEvent";
import { SitEvent } from "./events/SitEvent";
import { StartTrickEvent } from "./events/StartTrickEvent";
import { TurboHearts } from "./game/TurboHearts";
import "./styles/style.scss";
import { EventData, Card } from "./types";
import { YourPlayEvent } from "./events/YourPlayEvent";
import { StartPassingEvent } from "./events/StartPassingEvent";
import { StartChargingEvent } from "./events/StartChargingEvent";
import * as cookie from "cookie";
import { PlaySubmitter } from "./game/PlaySubmitter";

// TODO extract
function toEvent(th: TurboHearts, event: EventData) {
  switch (event.type) {
    case "sit":
      return new SitEvent(th, event);
    case "deal":
      return new DealEvent(th, event);
    case "start_passing":
      return new StartPassingEvent(th, event);
    case "send_pass":
      return new SendPassEvent(th, event);
    case "recv_pass":
      return new ReceivePassEvent(th, event);
    case "start_charging":
      return new StartChargingEvent(th, event);
    case "charge":
      return new ChargeEvent(th, event);
    case "start_trick":
      return new StartTrickEvent(th, event);
    case "your_play":
      event.legalPlays = (event as any).legal_plays;
      delete (event as any).legal_plays;
      return new YourPlayEvent(th, event);
    case "play":
      return new PlayEvent(th, event);
    case "end_trick":
      return new EndTrickEvent(th, event);
    default:
      return undefined;
  }
}

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
  const eventSource = new EventSource(`/game/subscribe/${gameId}`, {
    withCredentials: true
  });
  eventSource.addEventListener("message", event => {
    const realEvent = toEvent(th, JSON.parse(event.data) as EventData);
    if (realEvent !== undefined) {
      th.pushEvent(realEvent);
    }
  });
});
