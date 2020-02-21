import { TurboHearts } from "../game/TurboHearts";
import { Event, SitEventData, SitPlayer } from "../types";
import { getPlayerAccessor } from "./playerAccessors";
import { BOTTOM, TOP, RIGHT, LEFT } from "../const";
import { Nameplate } from "../ui/Nameplate";

interface PlayerAccessor {
  (event: SitEventData): SitPlayer;
}

const NORTH_ACCESSOR: PlayerAccessor = (event: SitEventData) => event.north;
const EAST_ACCESSOR: PlayerAccessor = (event: SitEventData) => event.east;
const SOUTH_ACCESSOR: PlayerAccessor = (event: SitEventData) => event.south;
const WEST_ACCESSOR: PlayerAccessor = (event: SitEventData) => event.west;

const eventAccessors: {
  [bottomSeat: string]: { [position: string]: PlayerAccessor };
} = {};
eventAccessors["north"] = {};
eventAccessors["north"]["top"] = SOUTH_ACCESSOR;
eventAccessors["north"]["right"] = WEST_ACCESSOR;
eventAccessors["north"]["bottom"] = NORTH_ACCESSOR;
eventAccessors["north"]["left"] = EAST_ACCESSOR;
eventAccessors["east"] = {};
eventAccessors["east"]["top"] = WEST_ACCESSOR;
eventAccessors["east"]["right"] = NORTH_ACCESSOR;
eventAccessors["east"]["bottom"] = EAST_ACCESSOR;
eventAccessors["east"]["left"] = SOUTH_ACCESSOR;
eventAccessors["south"] = {};
eventAccessors["south"]["top"] = NORTH_ACCESSOR;
eventAccessors["south"]["right"] = EAST_ACCESSOR;
eventAccessors["south"]["bottom"] = SOUTH_ACCESSOR;
eventAccessors["south"]["left"] = WEST_ACCESSOR;
eventAccessors["west"] = {};
eventAccessors["west"]["top"] = EAST_ACCESSOR;
eventAccessors["west"]["right"] = SOUTH_ACCESSOR;
eventAccessors["west"]["bottom"] = WEST_ACCESSOR;
eventAccessors["west"]["left"] = NORTH_ACCESSOR;

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
    const top = getPlayerAccessor(this.th.bottomSeat, "north")(this.th);
    const right = getPlayerAccessor(this.th.bottomSeat, "east")(this.th);
    const bottom = getPlayerAccessor(this.th.bottomSeat, "south")(this.th);
    const left = getPlayerAccessor(this.th.bottomSeat, "west")(this.th);
    top.name = eventAccessors[this.th.bottomSeat]["top"](this.event).name;
    top.type = eventAccessors[this.th.bottomSeat]["top"](this.event).type;
    right.name = eventAccessors[this.th.bottomSeat]["right"](this.event).name;
    right.type = eventAccessors[this.th.bottomSeat]["right"](this.event).type;
    bottom.name = eventAccessors[this.th.bottomSeat]["bottom"](this.event).name;
    bottom.type = eventAccessors[this.th.bottomSeat]["bottom"](this.event).type;
    left.name = eventAccessors[this.th.bottomSeat]["left"](this.event).name;
    left.type = eventAccessors[this.th.bottomSeat]["left"](this.event).type;
    const topName = new Nameplate(top.name, TOP.x, TOP.y + 30, 0);
    const rightName = new Nameplate(right.name, RIGHT.x + 4, RIGHT.y, -Math.PI / 2);
    const bottomName = new Nameplate(bottom.name, BOTTOM.x, BOTTOM.y + 3, 0);
    const leftName = new Nameplate(left.name, LEFT.x - 4, LEFT.y, Math.PI / 2);
    this.th.nameplates.push(topName);
    this.th.nameplates.push(rightName);
    this.th.nameplates.push(bottomName);
    this.th.nameplates.push(leftName);
  }

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
