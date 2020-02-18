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

const SERVER = "http://localhost:7380";
const GAME_ID = "99aff84a-8300-42a8-8744-dade0b731c47";

function playCard(card: Card) {
  return fetch(`${SERVER}/game/play`, {
    credentials: "include",
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ id: GAME_ID, card })
  });
}

function passCards(cards: Card[]) {
  return fetch(`${SERVER}/game/pass`, {
    credentials: "include",
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ id: GAME_ID, cards })
  });
}

(window as any).playCard = playCard;

document.addEventListener("DOMContentLoaded", event => {
  const th = new TurboHearts(document.getElementById("turbo-hearts") as HTMLCanvasElement, passCards, playCard);
  const eventSource = new EventSource(`${SERVER}/game/subscribe/${GAME_ID}`, {
    withCredentials: true
  });
  eventSource.addEventListener("message", event => {
    console.log(event);
    const realEvent = toEvent(th, JSON.parse(event.data) as EventData);
    if (realEvent !== undefined) {
      th.pushEvent(realEvent);
    }
  });
  // for (const event of HIDDEN_GAME) {
  //   const realEvent = toEvent(th, event as EventData);
  //   if (realEvent !== undefined) {
  //     th.pushEvent(realEvent);
  //   }
  // }
});
