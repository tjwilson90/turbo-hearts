import { EventEmitter, ListenerFn } from "eventemitter3";
import { ChargeStatusEventData, EventData, PassStatusEventData } from "../types";

export type EventType = "event" | EventData["type"];

function renameProp(object: any, from: string, to: string) {
  object[to] = object[from];
  delete object[from];
}

function mutateNesw<T extends PassStatusEventData | ChargeStatusEventData>(event: T) {
  const mutEvent = event as any;
  renameProp(mutEvent, "north_done", "northDone");
  renameProp(mutEvent, "east_done", "eastDone");
  renameProp(mutEvent, "south_done", "southDone");
  renameProp(mutEvent, "west_done", "westDone");
  return event;
}

function unrustify(event: EventData): EventData {
  switch (event.type) {
    case "sit":
      renameProp(event.north, "user_id", "userId");
      renameProp(event.east, "user_id", "userId");
      renameProp(event.south, "user_id", "userId");
      renameProp(event.west, "user_id", "userId");
      return event;
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
      return event;
    case "chat":
      renameProp(event, "user_id", "userId");
      return event;
    case "play_status":
      renameProp(event, "legal_plays", "legalPlays");
      renameProp(event, "next_player", "nextPlayer");
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

  constructor(private gameId: string) {}

  public connect() {
    this.eventSource = new EventSource(`/game/subscribe/${this.gameId}`, {
      withCredentials: true
    });
    this.eventSource.addEventListener("message", this.handleEvent);
  }

  public on(event: EventType, fn: (event: EventData) => void) {
    this.emitter.on(event, fn);
  }

  public off(event: EventType, fn: (event: EventData) => void) {
    this.emitter.off(event, fn);
  }

  public once(event: EventType, fn: (event: EventData) => void) {
    this.emitter.once(event, fn);
  }

  private handleEvent = (event: MessageEvent) => {
    const rawEvent: EventData = unrustify(JSON.parse(event.data) as EventData);
    this.emitter.emit("event", rawEvent);
    this.emitter.emit(rawEvent.type, rawEvent);
  };
}
