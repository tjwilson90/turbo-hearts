import { TurboHearts } from "../game/TurboHearts";
import { Event, PassStatusEventData } from "../types";
import { DirectionAccessor, seatEventFunction } from "./handPositions";
import { getPlayerAccessor } from "./playerAccessors";

const PassDoneAccessor: DirectionAccessor<PassStatusEventData, boolean> = {
  north: e => !e.northDone,
  east: e => !e.eastDone,
  south: e => !e.southDone,
  west: e => !e.westDone
};

export class PassStatusEvent implements Event {
  public type = "pass_status" as const;

  constructor(private th: TurboHearts, private event: PassStatusEventData) {}

  public begin() {
    const top = getPlayerAccessor(this.th.bottomSeat, "north")(this.th);
    const right = getPlayerAccessor(this.th.bottomSeat, "east")(this.th);
    const bottom = getPlayerAccessor(this.th.bottomSeat, "south")(this.th);
    const left = getPlayerAccessor(this.th.bottomSeat, "west")(this.th);
    top.toPlay = seatEventFunction(this.th.bottomSeat, "top", PassDoneAccessor, this.event);
    right.toPlay = seatEventFunction(this.th.bottomSeat, "right", PassDoneAccessor, this.event);
    bottom.toPlay = seatEventFunction(this.th.bottomSeat, "bottom", PassDoneAccessor, this.event);
    left.toPlay = seatEventFunction(this.th.bottomSeat, "left", PassDoneAccessor, this.event);
  }

  public transition(instant: boolean) {
    this.th.syncToPlay();
  }

  public isFinished() {
    return true;
  }
}
