import * as PIXI from "pixi.js";
import { Event, SitEventData } from "../types";
import { TurboHearts } from "../game/TurboHearts";
import { getPlayerAccessor } from "./playerAccessors";
import { TABLE_CENTER_X, TABLE_SIZE } from "../const";

export class SitEvent implements Event {
  constructor(private th: TurboHearts, private event: SitEventData) {}

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

    // const southLabel = new PIXI.Text(south.name);
    // southLabel.anchor.set(0.5);
    // southLabel.position.set(TABLE_CENTER_X, TABLE_SIZE - 20);
    // southLabel.zIndex = 100;
    // this.th.app.stage.addChild(southLabel);
  }

  public isFinished() {
    return true;
  }
}
