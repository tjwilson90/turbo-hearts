import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, StartTrickEventData } from "../types";

export class StartTrickEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: StartTrickEventData) {}

  public begin() {}

  public isFinished() {
    return true;
  }
}
