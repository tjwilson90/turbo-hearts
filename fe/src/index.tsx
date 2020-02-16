import { DealEvent } from "./events/DealEvent";
import { SendPassEvent } from "./events/SendPassEvent";
import { TurboHearts } from "./game/TurboHearts";
import "./styles/style.scss";
import { TEST_EVENTS } from "./test";
import { EventData } from "./types";
import { ReceivePassEvent } from "./events/ReceivePassEvent";
import { ChargeEvent } from "./events/ChargeEvent";
import { SitEvent } from "./events/SitEvent";
import { FULL_GAME } from "./fullGame";

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
