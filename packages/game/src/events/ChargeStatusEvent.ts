import { TurboHearts } from "../game/TurboHearts";
import { Event, ChargeStatusEventData } from "../types";
import { Button } from "../ui/Button";
import { CardPickSupport } from "./animations/CardPickSupport";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";
import { TABLE_SIZE, CARD_DISPLAY_HEIGHT, CARD_MARGIN } from "../const";
import { seatEventFunction, DirectionAccessor } from "./handPositions";

const ChargeDoneAccessor: DirectionAccessor<ChargeStatusEventData, boolean> = {
  north: e => !e.northDone,
  east: e => !e.eastDone,
  south: e => !e.southDone,
  west: e => !e.westDone
};

export class ChargeStatusEvent implements Event {
  public type = "charge_status" as const;

  private cardPickSupport: CardPickSupport;
  private button: Button;

  constructor(private th: TurboHearts, private event: ChargeStatusEventData) {}

  public begin() {
    const top = getPlayerAccessor(this.th.bottomSeat, "north")(this.th);
    const right = getPlayerAccessor(this.th.bottomSeat, "east")(this.th);
    const bottom = getPlayerAccessor(this.th.bottomSeat, "south")(this.th);
    const left = getPlayerAccessor(this.th.bottomSeat, "west")(this.th);
    top.toPlay = seatEventFunction(this.th.bottomSeat, "top", ChargeDoneAccessor, this.event);
    right.toPlay = seatEventFunction(this.th.bottomSeat, "right", ChargeDoneAccessor, this.event);
    bottom.toPlay = seatEventFunction(this.th.bottomSeat, "bottom", ChargeDoneAccessor, this.event);
    left.toPlay = seatEventFunction(this.th.bottomSeat, "left", ChargeDoneAccessor, this.event);
    this.th.syncToPlay();

    if (!this.isMyAction()) {
      return;
    }
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const chargeableCards = spriteCardsOf(player.cards, ["TC", "JD", "AH", "QS"]);
    this.cardPickSupport = new CardPickSupport(chargeableCards);
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
