import { TurboHearts } from "../game/TurboHearts";
import { Event, Pass, StartPassingEventData } from "../types";
import { Button } from "../ui/Button";
import { CardPickSupport } from "./animations/CardPickSupport";
import { getPlayerAccessor } from "./playerAccessors";
import { CARD_MARGIN, TABLE_SIZE, CARD_DISPLAY_HEIGHT } from "../const";

const directionText: { [P in Pass]: string } = {
  left: "Left",
  right: "Right",
  across: "Across",
  keeper: "In"
};

export class StartPassingEvent implements Event {
  public type = "start_passing" as const;

  private cardPickSupport: CardPickSupport;
  private button: Button;

  constructor(private th: TurboHearts, private event: StartPassingEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.cardPickSupport = new CardPickSupport(player.cards, "pass", this.setButtonEnabled);
    this.button = new Button(
      "Pass 3 Cards " + directionText[this.th.pass],
      TABLE_SIZE - CARD_DISPLAY_HEIGHT - CARD_MARGIN * 3,
      this.submitPass
    );
    this.setButtonEnabled();
    this.th.app.stage.addChild(this.button.container);
  }

  private setButtonEnabled = () => {
    this.button.setEnabled(this.cardPickSupport.picked.size === 3);
  };

  private submitPass = () => {
    this.button.setEnabled(false);
    this.cardPickSupport.cleanUp();
    this.th.app.stage.removeChild(this.button.container);
    this.th.submitter.passCards([...this.cardPickSupport.picked.values()].map(c => c.card));
  };

  public transition(instant: boolean) {}

  public isFinished() {
    // Passing is non-blocking.
    return true;
  }
}
