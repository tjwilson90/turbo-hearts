import { TurboHearts } from "../game/TurboHearts";
import { Event, YourChargeEventData } from "../types";
import { Button } from "../ui/Button";
import { CardPickSupport } from "./animations/CardPickSupport";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class YourChargeEvent implements Event {
  public type = "your_charge" as const;

  private cardPickSupport: CardPickSupport;
  private button: Button;

  constructor(private th: TurboHearts, private event: YourChargeEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const chargeableCards = spriteCardsOf(player.cards, ["TC", "JD", "AH", "QS"]);
    this.cardPickSupport = new CardPickSupport(chargeableCards);
    this.button = new Button("Charge Cards", this.submitCharge);
    this.button.setEnabled(true);
    this.th.app.stage.addChild(this.button.container);
    this.th.asyncEvent = this;
  }

  private submitCharge = () => {
    this.button.setEnabled(false);
    this.cardPickSupport.cleanUp();
    this.th.app.stage.removeChild(this.button.container);
    this.th.asyncEvent = undefined;
    this.th.submitter.chargeCards([...this.cardPickSupport.picked.values()].map(c => c.card));
  };

  public isFinished() {
    // Async (at least classic)
    return true;
  }
}
