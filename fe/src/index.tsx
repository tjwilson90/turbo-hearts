import { ChargeEvent } from "./events/ChargeEvent";
import { DealEvent } from "./events/DealEvent";
import { EndTrickEvent } from "./events/EndTrickEvent";
import { PlayEvent } from "./events/PlayEvent";
import { ReceivePassEvent } from "./events/ReceivePassEvent";
import { SendPassEvent } from "./events/SendPassEvent";
import { SitEvent } from "./events/SitEvent";
import { StartTrickEvent } from "./events/StartTrickEvent";
import { FULL_GAME } from "./fullGame";
import { TurboHearts } from "./game/TurboHearts";
import "./styles/style.scss";
import { EventData } from "./types";

function toEvent(th: TurboHearts, event: EventData) {
  switch (event.type) {
    case "sit":
      return new SitEvent(th, event);
    case "deal":
      return new DealEvent(th, event);
    case "send_pass":
      return new SendPassEvent(th, event);
    case "recv_pass":
      return new ReceivePassEvent(th, event);
    case "charge":
      return new ChargeEvent(th, event);
    case "start_trick":
      return new StartTrickEvent(th, event);
    case "play":
      return new PlayEvent(th, event);
    case "end_trick":
      return new EndTrickEvent(th, event);
    default:
      return undefined;
  }
}

document.addEventListener("DOMContentLoaded", event => {
  const th = new TurboHearts(document.getElementById("turbo-hearts") as HTMLCanvasElement);
  for (const event of FULL_GAME) {
    const realEvent = toEvent(th, event as EventData);
    if (realEvent !== undefined) {
      th.pushEvent(realEvent);
    }
  }
});
