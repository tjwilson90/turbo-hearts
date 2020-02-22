import { TurboHearts } from "../game/TurboHearts";
import { Event, PlayStatusEventData } from "../types";
import { CardPickSupport } from "./animations/CardPickSupport";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class PlayStatusEvent implements Event {
  public type = "play_status" as const;

  private finished = false;
  private cardPickSupport: CardPickSupport;

  constructor(private th: TurboHearts, private event: PlayStatusEventData) {}

  public begin() {
    if (this.event.nextPlayer !== this.th.bottomSeat) {
      return;
    }
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
