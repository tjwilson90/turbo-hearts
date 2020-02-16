import TWEEN from "@tweenjs/tween.js";
import { CHARGE_OVERLAP, FAST_ANIMATION_DELAY, FAST_ANIMATION_DURATION } from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { ChargeEventData, Event } from "../types";
import { pushAll, removeAll } from "../util/array";
import { groupCards } from "./groupCards";
import { getHandPosition } from "./handPositions";
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

    const handPosition = getHandPosition(this.th.bottomSeat, this.event.seat);
    const cardDests = groupCards(player.cards, handPosition.x, handPosition.y, handPosition.rotation);
    let delay = 0;
    let i = 0;
    for (const card of player.cards) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(cardDests[i], FAST_ANIMATION_DURATION)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );

      delay += FAST_ANIMATION_DELAY;
      i++;
    }
    const chargeDests = groupCards(
      player.chargedCards,
      handPosition.chargeX,
      handPosition.chargeY,
      handPosition.rotation,
      CHARGE_OVERLAP
    );
    delay = 0;
    i = 0;
    for (const card of player.chargedCards) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(chargeDests[i], FAST_ANIMATION_DURATION)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );

      delay += FAST_ANIMATION_DELAY;
      i++;
    }
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
