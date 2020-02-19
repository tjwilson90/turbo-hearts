import { TurboHearts } from "../game/TurboHearts";
import { Event, SitEventData } from "../types";
import { getPlayerAccessor } from "./playerAccessors";

export class SitEvent implements Event {
  public type = "sit" as const;

  constructor(private th: TurboHearts, private event: SitEventData) {
    if (event.north.name === th.userId) {
      th.bottomSeat = "north";
    } else if (event.east.name === th.userId) {
      th.bottomSeat = "east";
    } else if (event.south.name === th.userId) {
      th.bottomSeat = "south";
    } else if (event.west.name === th.userId) {
      th.bottomSeat = "west";
    } else {
      th.bottomSeat = "south";
    }
  }

  public begin() {
    const north = getPlayerAccessor(this.th.bottomSeat, "north")(this.th);
    const east = getPlayerAccessor(this.th.bottomSeat, "east")(this.th);
    const south = getPlayerAccessor(this.th.bottomSeat, "south")(this.th);
    const west = getPlayerAccessor(this.th.bottomSeat, "west")(this.th);
    north.name = this.event.north.name;
    north.type = this.event.north.type;
    east.name = this.event.east.name;
    east.type = this.event.east.type;
    south.name = this.event.south.name;
    south.type = this.event.south.type;
    west.name = this.event.west.name;
    west.type = this.event.west.type;
  }

  public isFinished() {
    return true;
  }
}
