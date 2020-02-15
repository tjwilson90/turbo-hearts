import TWEEN from "@tweenjs/tween.js";
import {
  BOTTOM,
  BOTTOM_LEFT,
  BOTTOM_RIGHT,
  LEFT,
  RIGHT,
  TOP,
  TOP_LEFT,
  TOP_RIGHT
} from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Event, SendPassData, SpriteCard } from "../types";
import { groupCards } from "./groupCards";

export class SendPassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: SendPassData) {}

  public begin() {
    const passDestination = this.getPassDestination();
    const cards = this.updateCards();
    let delay = 0;
    let i = 0;
    const duration = 300;
    const interval = 80;

    const cardDests = groupCards(
      cards.cardsToMove,
      passDestination.x,
      passDestination.y,
      passDestination.rotation
    );
    for (const card of cards.cardsToMove) {
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
          .to({ rotation: passDestination.rotation }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );

      delay += interval;
      i++;
    }
    const handDestination = this.getHandDestination();
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
          .to(
            {
              x: keepDests[i].x,
              y: keepDests[i].y
            },
            1000
          )
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      i++;
    }
  }

  private updateCards() {
    let hand: SpriteCard[];
    let setCards: (cards: SpriteCard[], limboCards: SpriteCard[]) => void;
    switch (this.event.from) {
      case "north":
        hand = this.th.topCards;
        setCards = (cards, limboCards) => {
          this.th.topCards = cards;
          this.th.topLimboCards = limboCards;
        };
        break;
      case "east":
        hand = this.th.rightCards;
        setCards = (cards, limboCards) => {
          this.th.rightCards = cards;
          this.th.rightLimboCards = limboCards;
        };
        break;
      case "south":
        hand = this.th.bottomCards;
        setCards = (cards, limboCards) => {
          this.th.bottomCards = cards;
          this.th.bottomLimboCards = limboCards;
        };
        break;
      case "west":
        hand = this.th.leftCards;
        setCards = (cards, limboCards) => {
          this.th.leftCards = cards;
          this.th.leftLimboCards = limboCards;
        };
        break;
    }
    if (this.event.cards.length === 0) {
      // pass hidden cards
      return { cardsToMove: [], cardsToKeep: [] };
    } else {
      const set = new Set(this.event.cards);
      const cardsToMove = hand.filter(c => set.has(c.card));
      const cardsToKeep = hand.filter(c => !set.has(c.card));
      setCards(cardsToKeep, cardsToMove);
      return { cardsToMove, cardsToKeep };
    }
  }

  private getHandDestination() {
    switch (this.event.from) {
      case "north":
        return TOP;
      case "east":
        return RIGHT;
      case "south":
        return BOTTOM;
      case "west":
        return LEFT;
    }
  }

  private getPassDestination() {
    switch (this.event.from) {
      case "north":
        switch (this.th.pass) {
          case "Left":
            return TOP_RIGHT;
        }
        break;
      case "east":
        switch (this.th.pass) {
          case "Left":
            return BOTTOM_RIGHT;
        }
        break;
      case "south":
        switch (this.th.pass) {
          case "Left":
            return BOTTOM_LEFT;
        }
        break;
      case "west":
        switch (this.th.pass) {
          case "Left":
            return TOP_LEFT;
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
