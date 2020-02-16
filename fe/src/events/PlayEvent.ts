import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, PlayEventData } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateHand, animatePlay } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class PlayEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: PlayEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.seat)(this.th);
    const cards = spriteCardsOf([...player.cards, ...player.chargedCards], [this.event.card]);

    pushAll(player.playCards, cards);
    removeAll(player.cards, cards);
    removeAll(player.chargedCards, cards);

    this.tweens.push(...animateHand(this.th, this.event.seat));
    this.tweens.push(...animatePlay(this.th, this.event.seat));
  }

  public isFinished() {
    for (const tween of this.tweens) {
      if (tween.isPlaying()) {
        return false;
      }
    }
    return true;
  }
}
