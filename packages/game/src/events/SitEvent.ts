import { LEFT, RIGHT, TOP, BOTTOM } from "../const";
import { TurboHearts, Player } from "../game/TurboHearts";
import { Event, SitEventData, SitPlayer } from "../types";
import { Nameplate } from "../ui/Nameplate";
import { getPlayerAccessor } from "./playerAccessors";
import { DirectionAccessor, seatEventFunction } from "./handPositions";

const NameTypeAccessor: DirectionAccessor<SitEventData, { name: string; type: "human" | "bot" }> = {
  north: e => ({ name: e.north.name, type: e.north.type }),
  east: e => ({ name: e.east.name, type: e.east.type }),
  south: e => ({ name: e.south.name, type: e.south.type }),
  west: e => ({ name: e.west.name, type: e.west.type })
};

const PlayerAccessor: DirectionAccessor<TurboHearts, Player> = {
  north: e => e.topPlayer,
  east: e => e.rightPlayer,
  south: e => e.bottomPlayer,
  west: e => e.leftPlayer
};

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
    // const top = seatEventFunction(this.th.bottomSeat, "top", PlayerAccessor, this.th);
    // const right = seatEventFunction(this.th.bottomSeat, "right", PlayerAccessor, this.th);
    // const bottom = seatEventFunction(this.th.bottomSeat, "bottom", PlayerAccessor, this.th);
    // const left = seatEventFunction(this.th.bottomSeat, "left", PlayerAccessor, this.th);
    const top = getPlayerAccessor(this.th.bottomSeat, "north")(this.th);
    const right = getPlayerAccessor(this.th.bottomSeat, "east")(this.th);
    const bottom = getPlayerAccessor(this.th.bottomSeat, "south")(this.th);
    const left = getPlayerAccessor(this.th.bottomSeat, "west")(this.th);

    const topNameType = seatEventFunction(this.th.bottomSeat, "top", NameTypeAccessor, this.event);
    top.name = topNameType.name;
    top.type = topNameType.type;
    top.nameplate.setName(top.name);
    const rightNameType = seatEventFunction(this.th.bottomSeat, "right", NameTypeAccessor, this.event);
    right.name = rightNameType.name;
    right.type = rightNameType.type;
    right.nameplate.setName(right.name);
    const bottomNameType = seatEventFunction(this.th.bottomSeat, "bottom", NameTypeAccessor, this.event);
    bottom.name = bottomNameType.name;
    bottom.type = bottomNameType.type;
    bottom.nameplate.setName(bottom.name);
    const leftNameType = seatEventFunction(this.th.bottomSeat, "left", NameTypeAccessor, this.event);
    left.name = leftNameType.name;
    left.type = leftNameType.type;
    left.nameplate.setName(left.name);
  }

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
