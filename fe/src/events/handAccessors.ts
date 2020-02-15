import { TurboHearts } from "../game/TurboHearts";
import { SpriteCard, Seat } from "../types";

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

export function getHandAccessor(th: TurboHearts, bottomSeat: Seat, seat: Seat) {
  return {
    getCards: () => handAccessors[bottomSeat][seat].getCards(th),
    getLimboCards: () => handAccessors[bottomSeat][seat].getLimboCards(th),
    setCards: (cards: SpriteCard[]) =>
      handAccessors[bottomSeat][seat].setCards(th, cards),
    setLimboCards: (cards: SpriteCard[]) =>
      handAccessors[bottomSeat][seat].setLimboCards(th, cards)
  };
}
