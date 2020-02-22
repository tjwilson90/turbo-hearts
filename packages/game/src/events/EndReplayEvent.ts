import { TurboHearts } from "../game/TurboHearts";
import { Event, EndReplayEventData } from "../types";

export class EndReplayEvent implements Event {
  public type = "start_trick" as const;

  constructor(private th: TurboHearts, private event: EndReplayEventData) {}

  public begin() {
    this.th.replay = false;
  }

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
