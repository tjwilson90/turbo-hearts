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
import { Event, SendPassData, SpriteCard, PointWithRotation } from "../types";
import { groupCards } from "./groupCards";

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

const handDestinations: {
  [bottomSeat: string]: { [passFrom: string]: PointWithRotation };
} = {};
handDestinations["north"] = {};
handDestinations["north"]["north"] = BOTTOM;
handDestinations["north"]["east"] = LEFT;
handDestinations["north"]["south"] = TOP;
handDestinations["north"]["west"] = RIGHT;
handDestinations["east"] = {};
handDestinations["east"]["north"] = RIGHT;
handDestinations["east"]["east"] = BOTTOM;
handDestinations["east"]["south"] = LEFT;
handDestinations["east"]["west"] = TOP;
handDestinations["south"] = {};
handDestinations["south"]["north"] = TOP;
handDestinations["south"]["east"] = RIGHT;
handDestinations["south"]["south"] = BOTTOM;
handDestinations["south"]["west"] = LEFT;
handDestinations["west"] = {};
handDestinations["west"]["north"] = LEFT;
handDestinations["west"]["east"] = TOP;
handDestinations["west"]["south"] = RIGHT;
handDestinations["west"]["west"] = BOTTOM;

interface HandAccessor {
  getCards: (th: TurboHearts) => SpriteCard[];
  getLimboCards: (th: TurboHearts) => SpriteCard[];
  setCards: (th: TurboHearts, cards: SpriteCard[]) => void;
  setLimboCards: (th: TurboHearts, cards: SpriteCard[]) => void;
}

const TOP_HAND_ACCESSOR: HandAccessor = {
  getCards: (th: TurboHearts) => th.topCards,
  getLimboCards: (th: TurboHearts) => th.topLimboCards,
  setCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.topCards = cards;
  },
  setLimboCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.topLimboCards = cards;
  }
};
const RIGHT_HAND_ACCESSOR: HandAccessor = {
  getCards: (th: TurboHearts) => th.rightCards,
  getLimboCards: (th: TurboHearts) => th.rightLimboCards,
  setCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.rightCards = cards;
  },
  setLimboCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.rightLimboCards = cards;
  }
};
const BOTTOM_HAND_ACCESSOR: HandAccessor = {
  getCards: (th: TurboHearts) => th.bottomCards,
  getLimboCards: (th: TurboHearts) => th.bottomLimboCards,
  setCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.bottomCards = cards;
  },
  setLimboCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.bottomLimboCards = cards;
  }
};
const LEFT_HAND_ACCESSOR: HandAccessor = {
  getCards: (th: TurboHearts) => th.leftCards,
  getLimboCards: (th: TurboHearts) => th.leftLimboCards,
  setCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.leftCards = cards;
  },
  setLimboCards: (th: TurboHearts, cards: SpriteCard[]) => {
    th.leftLimboCards = cards;
  }
};

const handAccessors: {
  [bottomSeat: string]: { [passFrom: string]: HandAccessor };
} = {};
handAccessors["north"] = {};
handAccessors["north"]["north"] = BOTTOM_HAND_ACCESSOR;
handAccessors["north"]["east"] = LEFT_HAND_ACCESSOR;
handAccessors["north"]["south"] = TOP_HAND_ACCESSOR;
handAccessors["north"]["west"] = RIGHT_HAND_ACCESSOR;
handAccessors["east"] = {};
handAccessors["east"]["north"] = RIGHT_HAND_ACCESSOR;
handAccessors["east"]["east"] = BOTTOM_HAND_ACCESSOR;
handAccessors["east"]["south"] = LEFT_HAND_ACCESSOR;
handAccessors["east"]["west"] = TOP_HAND_ACCESSOR;
handAccessors["south"] = {};
handAccessors["south"]["north"] = TOP_HAND_ACCESSOR;
handAccessors["south"]["east"] = RIGHT_HAND_ACCESSOR;
handAccessors["south"]["south"] = BOTTOM_HAND_ACCESSOR;
handAccessors["south"]["west"] = LEFT_HAND_ACCESSOR;
handAccessors["west"] = {};
handAccessors["west"]["north"] = LEFT_HAND_ACCESSOR;
handAccessors["west"]["east"] = TOP_HAND_ACCESSOR;
handAccessors["west"]["south"] = RIGHT_HAND_ACCESSOR;
handAccessors["west"]["west"] = BOTTOM_HAND_ACCESSOR;

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
    const handAccessor = handAccessors[this.th.bottomSeat][this.event.from];
    if (this.event.cards.length === 0) {
      // pass hidden cards
      return { cardsToMove: [], cardsToKeep: [] };
    } else {
      const set = new Set(this.event.cards);
      const hand = handAccessor.getCards(this.th);
      const cardsToMove = hand.filter(c => set.has(c.card));
      const cardsToKeep = hand.filter(c => !set.has(c.card));
      handAccessor.setCards(this.th, cardsToKeep);
      handAccessor.setLimboCards(this.th, cardsToMove);
      return { cardsToMove, cardsToKeep };
    }
  }

  private getHandDestination() {
    return handDestinations[this.th.bottomSeat][this.event.from];
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
