import { TurboHearts } from "../game/TurboHearts";
import { Event, YourPlayEventData } from "../types";
import { CardPickSupport } from "./animations/CardPickSupport";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class YourPlayEvent implements Event {
  public type = "your_play" as const;

  private finished = false;
  private cardPickSupport: CardPickSupport;

  constructor(private th: TurboHearts, private event: YourPlayEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const playableCards = [...player.cards, ...player.chargedCards];
    const legalPlays = spriteCardsOf(playableCards, this.event.legalPlays);
    this.cardPickSupport = new CardPickSupport(legalPlays, this.onPick);
  }

  private onPick = async () => {
    const cards = Array.from(this.cardPickSupport.picked.values());
    if (cards.length !== 1) {
      return;
    }
    this.cardPickSupport.cleanUp();
    await this.th.submitter.playCard(cards[0].card);
    this.finished = true;
  };

  public transition(instant: boolean) {}

  public isFinished() {
    return this.finished;
  }
}
