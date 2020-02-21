import { SitEvent } from "../events/SitEvent";
import { TurboHearts } from "./TurboHearts";
import { EventData } from "../types";
import { DealEvent } from "../events/DealEvent";
import { StartPassingEvent } from "../events/StartPassingEvent";
import { SendPassEvent } from "../events/SendPassEvent";
import { ReceivePassEvent } from "../events/ReceivePassEvent";
import { StartChargingEvent } from "../events/StartChargingEvent";
import { ChargeEvent } from "../events/ChargeEvent";
import { StartTrickEvent } from "../events/StartTrickEvent";
import { YourPlayEvent } from "../events/YourPlayEvent";
import { PlayEvent } from "../events/PlayEvent";
import { EndTrickEvent } from "../events/EndTrickEvent";
import { GameCompleteEvent } from "../events/GameCompleteEvent";
import { YourChargeEvent } from "../events/YourChargeEvent";

export class TurboHeartsEventSource {
  private eventSource: EventSource;
  private firstMessageReceived = false;

  constructor(private th: TurboHearts, gameId: string) {
    this.eventSource = new EventSource(`/game/subscribe/${gameId}`, {
      withCredentials: true
    });
    this.eventSource.addEventListener("message", this.handleEvent);
  }

  private convertEvent(event: EventData) {
    switch (event.type) {
      case "sit":
        return new SitEvent(this.th, event);
      case "deal":
        return new DealEvent(this.th, event);
      case "start_passing":
        return new StartPassingEvent(this.th, event);
      case "send_pass":
        return new SendPassEvent(this.th, event);
      case "recv_pass":
        return new ReceivePassEvent(this.th, event);
      case "start_charging":
        return new StartChargingEvent(this.th, event);
      case "your_charge":
        return new YourChargeEvent(this.th, event);
      case "charge":
        return new ChargeEvent(this.th, event);
      case "start_trick":
        return new StartTrickEvent(this.th, event);
      case "your_play":
        event.legalPlays = (event as any).legal_plays;
        delete (event as any).legal_plays;
        return new YourPlayEvent(this.th, event);
      case "play":
        return new PlayEvent(this.th, event);
      case "end_trick":
        return new EndTrickEvent(this.th, event);
      case "game_complete":
        return new GameCompleteEvent(this.th, event);
      default:
        return undefined;
    }
  }

  private handleEvent = (event: MessageEvent) => {
    if (!this.firstMessageReceived) {
      setTimeout(() => {
        this.th.replay = false;
      }, 500);
      this.firstMessageReceived = true;
    }
    console.log(event.data);
    const realEvent = this.convertEvent(JSON.parse(event.data) as EventData);
    if (realEvent !== undefined) {
      this.th.pushEvent(realEvent);
    }
    if (realEvent.type === "game_complete") {
      console.log("game_complete, disconnecting");
      this.eventSource.close();
    }
  };
}
