import { TurboHearts } from "../game/TurboHearts";
import { SendPassData, Event, SpriteCard } from "../types";
import TWEEN from "@tweenjs/tween.js";

const SIZE = 1000;
const INSET = 40;
const CARDS_LENGTH = 400;

export class SendPassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: SendPassData) {}

  public begin() {
    const dest = this.getDestination();
    const cards = this.getCards();
    let delay = 0;
    let i = 0;
    const duration = 300;
    const interval = 80;
    for (const card of cards) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(
            {
              x: dest.x + dest.offsetX * (i - 1),
              y: dest.y + dest.offsetY * (i - 1)
            },
            1000
          )
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.tweens.push(
        new TWEEN.Tween(card.sprite)
          .to({ rotation: dest.rotation }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      delay += interval;
      i++;
    }
  }

  private getCards() {
    let hand: SpriteCard[];
    switch (this.event.from) {
      case "north":
        hand = this.th.topCards;
        break;
      case "east":
        hand = this.th.rightCards;
        break;
      case "south":
        hand = this.th.bottomCards;
        break;
      case "west":
        hand = this.th.leftCards;
        break;
    }
    if (this.event.cards.length === 0) {
      // pass hidden cards
      return [];
    } else {
      const set = new Set(this.event.cards);
      const cardsToMove = hand.filter(c => set.has(c.card));
      return cardsToMove;
    }
  }

  private getDestination() {
    const r2o2 = Math.sqrt(2) / 2;
    switch (this.event.from) {
      case "north":
        switch (this.th.pass) {
          case "Left":
            return {
              x: SIZE - INSET * 4,
              y: INSET * 4,
              rotation: (Math.PI * 5) / 4,
              offsetX: r2o2 * 25,
              offsetY: r2o2 * 25
            };
        }
        break;
      case "east":
        switch (this.th.pass) {
          case "Left":
            return {
              x: SIZE - INSET * 4,
              y: SIZE - INSET * 4,
              rotation: (Math.PI * 3) / 4,
              offsetX: r2o2 * 25,
              offsetY: -r2o2 * 25
            };
        }
        break;
      case "south":
        switch (this.th.pass) {
          case "Left":
            return {
              x: INSET * 4,
              y: SIZE - INSET * 4,
              rotation: Math.PI / 4,
              offsetX: r2o2 * 25,
              offsetY: r2o2 * 25
            };
        }
        break;
      case "west":
        switch (this.th.pass) {
          case "Left":
            return {
              x: INSET * 4,
              y: INSET * 4,
              rotation: (Math.PI * 3) / 4,
              offsetX: r2o2 * 25,
              offsetY: -r2o2 * 25
            };
        }
        break;
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
