import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, ReceivePassData, SpriteCard } from "../types";
import { getHandAccessor } from "./handAccessors";
import { groupCards } from "./groupCards";
import { getHandPosition } from "./handPositions";
import { FAST_ANIMATION_DURATION, FAST_ANIMATION_DELAY } from "../const";

const limboSources: {
  [pass: string]: {
    [bottomSeat: string]: {
      [passFrom: string]: (th: TurboHearts) => SpriteCard[];
    };
  };
} = {};
limboSources["Left"] = {};
limboSources["Left"]["north"] = {};
limboSources["Left"]["north"]["north"] = (th: TurboHearts) =>
  th.rightLimboCards;
limboSources["Left"]["north"]["east"] = (th: TurboHearts) =>
  th.bottomLimboCards;
limboSources["Left"]["north"]["south"] = (th: TurboHearts) => th.leftLimboCards;
limboSources["Left"]["north"]["west"] = (th: TurboHearts) => th.topLimboCards;
limboSources["Left"]["east"] = {};
limboSources["Left"]["east"]["north"] = (th: TurboHearts) => th.topLimboCards;
limboSources["Left"]["east"]["east"] = (th: TurboHearts) => th.rightLimboCards;
limboSources["Left"]["east"]["south"] = (th: TurboHearts) =>
  th.bottomLimboCards;
limboSources["Left"]["east"]["west"] = (th: TurboHearts) => th.leftLimboCards;
limboSources["Left"]["south"] = {};
limboSources["Left"]["south"]["north"] = (th: TurboHearts) => th.leftLimboCards;
limboSources["Left"]["south"]["east"] = (th: TurboHearts) => th.topLimboCards;
limboSources["Left"]["south"]["south"] = (th: TurboHearts) =>
  th.rightLimboCards;
limboSources["Left"]["south"]["west"] = (th: TurboHearts) =>
  th.bottomLimboCards;
limboSources["Left"]["west"] = {};
limboSources["Left"]["west"]["north"] = (th: TurboHearts) =>
  th.bottomLimboCards;
limboSources["Left"]["west"]["east"] = (th: TurboHearts) => th.leftLimboCards;
limboSources["Left"]["west"]["south"] = (th: TurboHearts) => th.topLimboCards;
limboSources["Left"]["west"]["west"] = (th: TurboHearts) => th.rightLimboCards;

export class ReceivePassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: ReceivePassData) {}

  public begin() {
    const handAccessor = getHandAccessor(
      this.th,
      this.th.bottomSeat,
      this.event.to
    );
    const cards = handAccessor.getCards();
    this.updateCards(cards);

    const handPosition = getHandPosition(this.th.bottomSeat, this.event.to);
    const cardDests = groupCards(
      cards,
      handPosition.x,
      handPosition.y,
      handPosition.rotation
    );
    let delay = 0;
    let i = 0;
    for (const card of cards) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(
            {
              x: cardDests[i].x,
              y: cardDests[i].y
            },
            1000
          )
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.tweens.push(
        new TWEEN.Tween(card.sprite)
          .to({ rotation: handPosition.rotation }, FAST_ANIMATION_DURATION)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );

      delay += FAST_ANIMATION_DELAY;
      i++;
    }
  }

  private updateCards(hand: SpriteCard[]) {
    const limboSource = limboSources[this.th.pass][this.th.bottomSeat][
      this.event.to
    ](this.th);
    while (limboSource.length > 0) {
      // Note: this is mutating both hand and limbo arrays
      hand.push(limboSource.pop());
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
