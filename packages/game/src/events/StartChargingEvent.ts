import { TurboHearts } from "../game/TurboHearts";
import { Event, StartChargingEventData } from "../types";

export class StartChargingEvent implements Event {
  public type = "start_charging" as const;

  constructor(private th: TurboHearts, private event: StartChargingEventData) {}

  public begin() {}

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
