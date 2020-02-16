import { TurboHearts } from "../game/TurboHearts";
import { Event, PlayEventData } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateHand, animatePlay } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class PlayEvent implements Event {
  private finished = false;

  constructor(private th: TurboHearts, private event: PlayEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.seat)(this.th);
    const cards = spriteCardsOf([...player.cards, ...player.chargedCards], [this.event.card]);
    cards[0].sprite.zIndex = this.th.playIndex++ + 100;

    pushAll(player.playCards, cards);
    removeAll(player.cards, cards);
    removeAll(player.chargedCards, cards);

    Promise.all([animateHand(this.th, this.event.seat), animatePlay(this.th, this.event.seat)]).then(() => {
      this.finished = true;
    });
  }

  public isFinished() {
    return this.finished;
  }
}
