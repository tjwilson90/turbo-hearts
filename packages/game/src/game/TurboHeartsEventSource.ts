import { SitEvent } from "../events/SitEvent";
import { TurboHearts } from "./TurboHearts";
import { EventData, PassStatusEventData, ChargeStatusEventData, ChatEvent } from "../types";
import { DealEvent } from "../events/DealEvent";
import { StartPassingEvent } from "../events/StartPassingEvent";
import { SendPassEvent } from "../events/SendPassEvent";
import { ReceivePassEvent } from "../events/ReceivePassEvent";
import { StartChargingEvent } from "../events/StartChargingEvent";
import { ChargeEvent } from "../events/ChargeEvent";
import { StartTrickEvent } from "../events/StartTrickEvent";
import { PlayStatusEvent } from "../events/PlayStatusEvent";
import { PlayEvent } from "../events/PlayEvent";
import { EndTrickEvent } from "../events/EndTrickEvent";
import { GameCompleteEvent } from "../events/GameCompleteEvent";
import { ChargeStatusEvent } from "../events/ChargeStatusEvent";
import { PassStatusEvent } from "../events/PassStatusEvent";
import { EndReplayEvent } from "../events/EndReplayEvent";

function mutateNesw<T extends PassStatusEventData | ChargeStatusEventData>(event: T) {
  const mutEvent = event as any;
  event.northDone = mutEvent.north_done;
  event.eastDone = mutEvent.east_done;
  event.southDone = mutEvent.south_done;
  event.westDone = mutEvent.west_done;
  delete mutEvent.north_done;
  delete mutEvent.east_done;
  delete mutEvent.south_done;
  delete mutEvent.west_done;
  return event;
}

export class TurboHeartsEventSource {
  private eventSource: EventSource;

  constructor(private th: TurboHearts, gameId: string, private onChat: (chat: ChatEvent) => void) {
    this.eventSource = new EventSource(`/game/subscribe/${gameId}`, {
      withCredentials: true
    });
    this.eventSource.addEventListener("message", this.handleEvent);
  }

  private convertEvent(event: EventData) {
    switch (event.type) {
      case "sit":
        return new SitEvent(this.th, event);
      case "end_replay":
        return new EndReplayEvent(this.th, event);
      case "deal":
        return new DealEvent(this.th, event);
      case "start_passing":
        return new StartPassingEvent(this.th, event);
      case "pass_status":
        return new PassStatusEvent(this.th, mutateNesw(event));
      case "send_pass":
        return new SendPassEvent(this.th, event);
      case "recv_pass":
        return new ReceivePassEvent(this.th, event);
      case "start_charging":
        return new StartChargingEvent(this.th, event);
      case "charge_status":
        return new ChargeStatusEvent(this.th, mutateNesw(event));
      case "charge":
        return new ChargeEvent(this.th, event);
      case "start_trick":
        return new StartTrickEvent(this.th, event);
      case "play_status":
        const mutEvent = event as any;
        event.legalPlays = mutEvent.legal_plays;
        event.nextPlayer = mutEvent.next_player;
        delete (event as any).legal_plays;
        delete (event as any).next_player;
        return new PlayStatusEvent(this.th, event);
      case "play":
        return new PlayEvent(this.th, event);
      case "end_trick":
        return new EndTrickEvent(this.th, event);
      case "game_complete":
        return new GameCompleteEvent(this.th, event);
      case "chat":
        this.onChat(event);
      default:
        return undefined;
    }
  }

  private handleEvent = (event: MessageEvent) => {
    console.log(event.data);
    const realEvent = this.convertEvent(JSON.parse(event.data) as EventData);
    if (realEvent === undefined) {
      return;
    }
    this.th.pushEvent(realEvent);
  };
}
