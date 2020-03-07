import { EventEmitter } from "eventemitter3";
import { EventData, Seat } from "../types";
import { TurboHearts } from "./stateSnapshot";
import { getBottomSeat } from "../state/UserDispatcher";

export class PassTracker {
  private emitter = new EventEmitter();
  private seat: Seat | undefined;
  private firstTrickPlayed = false;
  private localPass: TurboHearts.LocalPass = { sent: undefined, received: undefined };

  constructor(private userId: string) {}

  public onEvent = (event: EventData) => {
    switch (event.type) {
      case "sit": {
        this.seat = getBottomSeat(event, this.userId);
        break;
      }

      case "deal": {
        this.firstTrickPlayed = false;
        break;
      }

      case "send_pass": {
        if (event.from === this.seat) {
          this.localPass = { ...this.localPass, sent: event.cards };
          this.emitter.emit("pass", this.localPass);
        }
        break;
      }

      case "recv_pass": {
        if (event.to === this.seat) {
          this.localPass = { ...this.localPass, received: event.cards };
          this.emitter.emit("pass", this.localPass);
        }
        break;
      }

      case "end_trick": {
        if (!this.firstTrickPlayed) {
          this.localPass = { sent: undefined, received: undefined };
          this.emitter.emit("pass", undefined);
          this.firstTrickPlayed = true;
        }
        break;
      }
    }
  };

  public on(event: "pass", fn: (next: TurboHearts.LocalPass) => void) {
    this.emitter.on(event, fn);
  }

  public off(event: "pass", fn: (next: TurboHearts.LocalPass) => void) {
    this.emitter.off(event, fn);
  }
}
