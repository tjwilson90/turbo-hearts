import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { ChargeEventData, Event } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateCharges, animatePlay, animateHand } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class ChargeEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: ChargeEventData) {}

  public begin() {
    if (this.event.cards.length === 0) {
      return;
    }
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.seat)(this.th);

    const chargeCards = spriteCardsOf(player.cards, this.event.cards);
    removeAll(player.cards, chargeCards);
    pushAll(player.chargedCards, chargeCards);

    this.tweens.push(...animateHand(this.th, this.event.seat));
    this.tweens.push(...animateCharges(this.th, this.event.seat));
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
