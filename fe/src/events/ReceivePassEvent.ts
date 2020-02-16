import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, ReceivePassEventData, SpriteCard } from "../types";
import { groupCards } from "./groupCards";
import { getHandPosition } from "./handPositions";
import { FAST_ANIMATION_DURATION, FAST_ANIMATION_DELAY } from "../const";
import { getPlayerAccessor } from "./playerAccessors";

const limboSources: {
  [pass: string]: {
    [bottomSeat: string]: {
      [passFrom: string]: (th: TurboHearts) => SpriteCard[];
    };
  };
} = {};
limboSources["Left"] = {};
limboSources["Left"]["north"] = {};
limboSources["Left"]["north"]["north"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["north"]["east"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["north"]["south"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["north"]["west"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["east"] = {};
limboSources["Left"]["east"]["north"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["east"]["east"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["east"]["south"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["east"]["west"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["south"] = {};
limboSources["Left"]["south"]["north"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["south"]["east"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["south"]["south"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["south"]["west"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["west"] = {};
limboSources["Left"]["west"]["north"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["west"]["east"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["west"]["south"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["west"]["west"] = (th: TurboHearts) => th.rightPlayer.limboCards;

export class ReceivePassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: ReceivePassEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.to)(this.th);
    const cards = player.cards;
    this.updateCards(cards);

    const handPosition = getHandPosition(this.th.bottomSeat, this.event.to);
    const cardDests = groupCards(cards, handPosition.x, handPosition.y, handPosition.rotation);
    let delay = 0;
    let i = 0;
    for (const card of cards) {
      card.sprite.zIndex = i;
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(cardDests[i], FAST_ANIMATION_DURATION)
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
    // TODO: this is resulting in jarring card flip
    this.th.app.stage.sortChildren();
  }

  private updateCards(hand: SpriteCard[]) {
    const limboSource = limboSources[this.th.pass][this.th.bottomSeat][this.event.to](this.th);
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
