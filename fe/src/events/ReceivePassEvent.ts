import { TurboHearts } from "../game/TurboHearts";
import { Event, ReceivePassData } from "../types";

export class ReceivePassEvent implements Event {
  constructor(private th: TurboHearts, private event: ReceivePassData) {}
  public begin() {}

  public isFinished() {
    return true;
  }
}
