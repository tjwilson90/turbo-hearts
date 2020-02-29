import { TurboHearts } from "../game/TurboHearts";
import { Event, ChargeStatusEventData } from "../types";
import { Button } from "../ui/Button";
import { CardPickSupport } from "./animations/CardPickSupport";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";
import { TABLE_SIZE, CARD_DISPLAY_HEIGHT, CARD_MARGIN } from "../const";

export class ChargeStatusEvent implements Event {
  public type = "charge_status" as const;

  private cardPickSupport: CardPickSupport;
  private button: Button;

  constructor(private th: TurboHearts, private event: ChargeStatusEventData) {}

  public begin() {
    if (!this.isMyAction()) {
      return;
    }
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const chargeableCards = spriteCardsOf(player.cards, ["TC", "JD", "AH", "QS"]);
    this.cardPickSupport = new CardPickSupport(chargeableCards, "charge");
    this.button = new Button("Charge Cards", TABLE_SIZE - CARD_DISPLAY_HEIGHT * 1.5, this.submitCharge);
    this.button.setEnabled(true);
    this.th.app.stage.addChild(this.button.container);
    this.th.app.stage.sortChildren();
    this.th.asyncEvent = this;
  }

  private isMyAction() {
    if (this.th.asyncEvent?.type == this.type) {
      return false;
    }
    switch (this.th.bottomSeat) {
      case "north":
        return !this.event.northDone;
      case "east":
        return !this.event.eastDone;
      case "south":
        return !this.event.southDone;
      case "west":
        return !this.event.westDone;
    }
  }

  private submitCharge = () => {
    this.button.setEnabled(false);
    this.cardPickSupport.cleanUp();
    this.th.app.stage.removeChild(this.button.container);
    this.th.asyncEvent = undefined;
    this.th.submitter.chargeCards([...this.cardPickSupport.picked.values()].map(c => c.card));
  };

  public transition(instant: boolean) {}

  public isFinished() {
    // Async (at least classic)
    return true;
  }
}
