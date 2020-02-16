import { TurboHearts } from "../game/TurboHearts";
import { EndTrickEventData, Event, Seat } from "../types";
import { getPlayerAccessor } from "./playerAccessors";

export class EndTrickEvent implements Event {
  constructor(private th: TurboHearts, private event: EndTrickEventData) {}

  public begin() {
    ["north", "east", "south", "west"].forEach((seat: Seat) => {
      const player = getPlayerAccessor(this.th.bottomSeat, seat)(this.th);
      player.playCards.forEach(card => {
        card.sprite.parent.removeChild(card.sprite);
      });
      player.playCards = [];
    });
    this.th.playIndex = 0;
  }

  public isFinished() {
    return true;
  }
}
