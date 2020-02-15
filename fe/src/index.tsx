import { DealEvent } from "./events/DealEvent";
import { SendPassEvent } from "./events/SendPassEvent";
import { TurboHearts } from "./game/TurboHearts";
import "./styles/style.scss";
import { TEST_EVENTS } from "./test";
import { EventData } from "./types";

function toEvent(th: TurboHearts, event: EventData) {
  switch (event.type) {
    case "deal":
      return new DealEvent(th, event);
    case "send_pass":
      return new SendPassEvent(th, event);
    default:
      return undefined;
  }
}

document.addEventListener("DOMContentLoaded", event => {
  const th = new TurboHearts(
    document.getElementById("turbo-hearts") as HTMLCanvasElement
  );
  // const events = [...TEST_EVENTS];
  for (const event of TEST_EVENTS) {
    const realEvent = toEvent(th, event as EventData);
    if (realEvent !== undefined) {
      th.pushEvent(realEvent);
    }
  }
});
