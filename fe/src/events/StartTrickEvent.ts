import { TurboHearts } from "../game/TurboHearts";
import { Event, StartTrickEventData } from "../types";

export class StartTrickEvent implements Event {
  constructor(private th: TurboHearts, private event: StartTrickEventData) {}

  public begin() {}

  public isFinished() {
    return true;
  }
}
