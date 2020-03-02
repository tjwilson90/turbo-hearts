import { EventEmitter } from "eventemitter3";
import { EventData } from "../types";
import { TurboHearts } from "./stateSnapshot";

export class TrickTracker {
  private emitter = new EventEmitter();
  private trickNumber = 0;
  private trick: Partial<TurboHearts.Trick> = {};

  public onEvent = (event: EventData) => {
    switch (event.type) {
      case "start_trick": {
        this.trick = {
          trickNumber: this.trickNumber,
          leader: event.leader,
          plays: []
        };
        break;
      }

      case "play": {
        this.trick.plays!.push(event.card);
        break;
      }

      case "end_trick": {
        this.trick.winner = event.winner;
        this.trickNumber++;
        this.emitter.emit("trick", this.trick);
        break;
      }

      case "start_passing": {
        this.trickNumber = 0;
        this.emitter.emit("reset");
        break;
      }
    }
  };

  public on(event: "trick" | "reset", fn: (event: { next: TurboHearts.Trick; previous: TurboHearts.Trick }) => void) {
    this.emitter.on(event, fn);
  }

  public off(event: "trick" | "reset", fn: (event: { next: TurboHearts.Trick; previous: TurboHearts.Trick }) => void) {
    this.emitter.off(event, fn);
  }
}
