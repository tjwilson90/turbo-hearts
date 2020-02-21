import { TurboHearts } from "../game/TurboHearts";
import { Event, GameCompleteEventData } from "../types";

export class GameCompleteEvent implements Event {
  public type = "game_complete" as const;

  constructor(private th: TurboHearts, private event: GameCompleteEventData) {}

  public begin() {
    this.th.resetForDeal();
  }

  public transition(instant: boolean) {}

  public isFinished() {
    return true;
  }
}
