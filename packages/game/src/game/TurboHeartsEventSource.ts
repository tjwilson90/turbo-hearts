import { EventEmitter, ListenerFn } from "eventemitter3";
import { ChargeStatusEventData, EventData, PassStatusEventData } from "../types";
import { TurboHearts } from "./TurboHearts";

export type EventType = "event" | EventData["type"];

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

function unrustify(event: EventData): EventData {
  switch (event.type) {
    case "sit":
    case "end_replay":
    case "deal":
    case "start_passing":
    case "send_pass":
    case "recv_pass":
    case "start_charging":
    case "charge":
    case "start_trick":
    case "play":
    case "end_trick":
    case "game_complete":
    case "chat":
      return event;
    case "play_status":
      const mutEvent = event as any;
      event.legalPlays = mutEvent.legal_plays;
      event.nextPlayer = mutEvent.next_player;
      delete (event as any).legal_plays;
      delete (event as any).next_player;
      return event;
    case "charge_status":
    case "pass_status":
      return mutateNesw(event);
    default:
      return event;
  }
}

export class TurboHeartsEventSource {
  private eventSource: EventSource;

  private emitter = new EventEmitter();

  constructor(private th: TurboHearts, gameId: string) {
    this.eventSource = new EventSource(`/game/subscribe/${gameId}`, {
      withCredentials: true
    });
    this.eventSource.addEventListener("message", this.handleEvent);
  }

  public on(event: EventType, fn: ListenerFn) {
    this.emitter.on(event, fn);
  }

  public off(event: EventType, fn: ListenerFn) {
    this.emitter.off(event, fn);
  }

  private handleEvent = (event: MessageEvent) => {
    const rawEvent: EventData = unrustify(JSON.parse(event.data) as EventData);
    this.emitter.emit("event", rawEvent);
    this.emitter.emit(rawEvent.type, rawEvent);
  };
}
