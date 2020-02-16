import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { EndTrickEventData, Event, Seat } from "../types";
import { getPlayerAccessor } from "./playerAccessors";

export class EndTrickEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: EndTrickEventData) {}

  public begin() {
    ["north", "east", "south", "west"].forEach((seat: Seat) => {
      const player = getPlayerAccessor(this.th.bottomSeat, seat)(this.th);
      player.playCards.forEach(card => {
        card.sprite.parent.removeChild(card.sprite);
      });
      player.playCards = [];
    });
  }

  public isFinished() {
    return true;
  }
}
