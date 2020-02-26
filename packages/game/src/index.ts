import * as cookie from "cookie";
import { PlaySubmitter } from "./game/PlaySubmitter";
import { TurboHearts } from "./game/TurboHearts";
import { TurboHeartsEventSource } from "./game/TurboHeartsEventSource";
import "./styles/style.scss";
import { ChatInput } from "./chat/ChatInput";
import { ChatEvent, EventData } from "./types";
import { SitEvent } from "./events/SitEvent";
import { EndReplayEvent } from "./events/EndReplayEvent";
import { DealEvent } from "./events/DealEvent";
import { StartPassingEvent } from "./events/StartPassingEvent";
import { PassStatusEvent } from "./events/PassStatus";
import { SendPassEvent } from "./events/SendPassEvent";
import { ReceivePassEvent } from "./events/ReceivePassEvent";
import { StartChargingEvent } from "./events/StartChargingEvent";
import { ChargeStatusEvent } from "./events/ChargeStatusEvent";
import { ChargeEvent } from "./events/ChargeEvent";
import { StartTrickEvent } from "./events/StartTrickEvent";
import { PlayStatusEvent } from "./events/PlayStatusEvent";
import { PlayEvent } from "./events/PlayEvent";
import { EndTrickEvent } from "./events/EndTrickEvent";
import { GameCompleteEvent } from "./events/GameCompleteEvent";

document.addEventListener("DOMContentLoaded", () => {
  const userId = cookie.parse(document.cookie)["NAME"];
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
  (window as any).th = th;
  const chatLog = document.getElementById("chat-log")!;
  const chatAppender = (message: ChatEvent) => {
    const div = document.createElement("div");
    div.classList.add("chat-message-container");
    const nameSpan = document.createElement("span");
    nameSpan.classList.add("chat-user");
    nameSpan.textContent = message.name;
    div.appendChild(nameSpan);
    const messageSpan = document.createElement("span");
    messageSpan.classList.add("chat-message");
    messageSpan.textContent = message.message;
    div.appendChild(messageSpan);
    chatLog.appendChild(div);
    div.scrollIntoView();
  };

  const eventSource = new TurboHeartsEventSource(th, gameId);
  eventSource.on("chat", chatAppender);

  eventSource.on("event", event => console.log(event));

  function convertEvent(th: TurboHearts, event: EventData) {
    switch (event.type) {
      case "sit":
        return new SitEvent(th, event);
      case "end_replay":
        return new EndReplayEvent(th, event);
      case "deal":
        return new DealEvent(th, event);
      case "start_passing":
        return new StartPassingEvent(th, event);
      case "pass_status":
        return new PassStatusEvent(th, event);
      case "send_pass":
        return new SendPassEvent(th, event);
      case "recv_pass":
        return new ReceivePassEvent(th, event);
      case "start_charging":
        return new StartChargingEvent(th, event);
      case "charge_status":
        return new ChargeStatusEvent(th, event);
      case "charge":
        return new ChargeEvent(th, event);
      case "start_trick":
        return new StartTrickEvent(th, event);
      case "play_status":
        return new PlayStatusEvent(th, event);
      case "play":
        return new PlayEvent(th, event);
      case "end_trick":
        return new EndTrickEvent(th, event);
      case "game_complete":
        return new GameCompleteEvent(th, event);
      case "chat":
      default:
        return undefined;
    }
  }

  eventSource.on("event", event => {
    const realEvent = convertEvent(th, event);
    if (realEvent === undefined) {
      return;
    }
    th.pushEvent(realEvent);
  });

  new ChatInput(document.getElementById("chat-input") as HTMLTextAreaElement, gameId);
});
