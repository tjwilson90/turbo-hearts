import { TurboHearts } from "../game/TurboHearts";
import { Event, StartTrickEventData } from "../types";

export class StartTrickEvent implements Event {
  public type = "start_trick" as const;

  constructor(private th: TurboHearts, private event: StartTrickEventData) {}

  public begin() {}

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
