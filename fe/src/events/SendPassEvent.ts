import TWEEN from "@tweenjs/tween.js";
import {
  BOTTOM_LEFT,
  BOTTOM_RIGHT,
  TOP_LEFT,
  TOP_RIGHT,
  FAST_ANIMATION_DELAY,
  FAST_ANIMATION_DURATION
} from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Event, PointWithRotation, SendPassData } from "../types";
import { groupCards } from "./groupCards";
import { getHandAccessor } from "./handAccessors";
import { getHandPosition } from "./handPositions";

const passDestinations: {
  [pass: string]: {
    [bottomSeat: string]: { [passFrom: string]: PointWithRotation };
  };
} = {};
passDestinations["Left"] = {};
passDestinations["Left"]["north"] = {};
passDestinations["Left"]["north"]["north"] = BOTTOM_LEFT;
passDestinations["Left"]["north"]["east"] = TOP_LEFT;
passDestinations["Left"]["north"]["south"] = TOP_RIGHT;
passDestinations["Left"]["north"]["west"] = BOTTOM_RIGHT;
passDestinations["Left"]["east"] = {};
passDestinations["Left"]["east"]["north"] = BOTTOM_RIGHT;
passDestinations["Left"]["east"]["east"] = BOTTOM_LEFT;
passDestinations["Left"]["east"]["south"] = TOP_LEFT;
passDestinations["Left"]["east"]["west"] = TOP_RIGHT;
passDestinations["Left"]["south"] = {};
passDestinations["Left"]["south"]["north"] = TOP_RIGHT;
passDestinations["Left"]["south"]["east"] = BOTTOM_RIGHT;
passDestinations["Left"]["south"]["south"] = BOTTOM_LEFT;
passDestinations["Left"]["south"]["west"] = TOP_LEFT;
passDestinations["Left"]["west"] = {};
passDestinations["Left"]["west"]["north"] = TOP_LEFT;
passDestinations["Left"]["west"]["east"] = TOP_RIGHT;
passDestinations["Left"]["west"]["south"] = BOTTOM_RIGHT;
passDestinations["Left"]["west"]["west"] = BOTTOM_LEFT;

export class SendPassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: SendPassData) {}

  public begin() {
    const passDestination = this.getPassDestination();
    const cards = this.updateCards();
    let delay = 0;
    let i = 0;

    const cardDests = groupCards(
      cards.cardsToMove,
      passDestination.x,
      passDestination.y,
      passDestination.rotation
    );
    for (const card of cards.cardsToMove) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(cardDests[i], FAST_ANIMATION_DURATION)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.tweens.push(
        new TWEEN.Tween(card.sprite)
          .to({ rotation: passDestination.rotation }, FAST_ANIMATION_DURATION)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );

      delay += FAST_ANIMATION_DELAY;
      i++;
    }
    const handDestination = getHandPosition(
      this.th.bottomSeat,
      this.event.from
    );
    const keepDests = groupCards(
      cards.cardsToKeep,
      handDestination.x,
      handDestination.y,
      handDestination.rotation
    );
    i = 0;
    for (const card of cards.cardsToKeep) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(keepDests[i], 1000)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      i++;
    }
  }

  private updateCards() {
    const handAccessor = getHandAccessor(
      this.th,
      this.th.bottomSeat,
      this.event.from
    );
    if (this.event.cards.length === 0) {
      // TODO pass hidden cards
      return { cardsToMove: [], cardsToKeep: [] };
    } else {
      const set = new Set(this.event.cards);
      const hand = handAccessor.getCards();
      const cardsToMove = hand.filter(c => set.has(c.card));
      const cardsToKeep = hand.filter(c => !set.has(c.card));
      handAccessor.setCards(cardsToKeep);
      handAccessor.setLimboCards(cardsToMove);
      return { cardsToMove, cardsToKeep };
    }
  }

  private getPassDestination() {
    return passDestinations[this.th.pass][this.th.bottomSeat][this.event.from];
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
