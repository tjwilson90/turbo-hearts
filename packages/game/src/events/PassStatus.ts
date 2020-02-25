import { TurboHearts } from "../game/TurboHearts";
import { Event, PassStatusEventData } from "../types";

export class PassStatusEvent implements Event {
  public type = "pass_status" as const;

  constructor(private th: TurboHearts, private event: PassStatusEventData) {}

  public begin() {}

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
