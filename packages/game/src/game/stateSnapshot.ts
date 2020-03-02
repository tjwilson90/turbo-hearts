import { Card, EventData, Pass } from "../types";
import { emptyArray } from "../util/array";
import { sortCards } from "./sortCards";

export const EMPTY_HAND: Card[] = [
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK"
];

const EMPTY_PASS: Card[] = ["BACK", "BACK", "BACK"];

export function cardsOf(cards: Card[], rawCards: Card[]) {
  const set = new Set(rawCards);
  return cards.filter(c => set.has(c));
}

export function notCardsOf(cards: Card[], rawCards: Card[]) {
  const set = new Set(rawCards);
  return cards.filter(c => !set.has(c));
}

export type Action = "none" | "pass" | "charge" | "play";

export namespace TurboHearts {
  export interface Player {
    type: "bot" | "human";
    userId: string;
    hand: Card[];
    plays: Card[];
    pile: Card[];
    limbo: Card[];
    charged: Card[];
    legalPlays: Card[];
    action: Action;
  }

  export interface StateSnapshot {
    index: number;
    event: EventData;
    north: Player;
    east: Player;
    south: Player;
    west: Player;

    pass: Pass;
    userId: string;
    handNumber: number;
    trickNumber: number;
    playNumber: number;
  }

  export interface Game {
    snapshots: StateSnapshot[];
  }
}

export function newPlayer(type: "bot" | "human", userId: string): TurboHearts.Player {
  return {
    type,
    userId,
    hand: emptyArray(),
    plays: emptyArray(),
    pile: emptyArray(),
    limbo: emptyArray(),
    charged: emptyArray(),
    legalPlays: emptyArray(),
    action: "none"
  };
}

export function withDeal(player: TurboHearts.Player, cards: Card[]): TurboHearts.Player {
  sortCards(cards);
  return {
    ...player,
    hand: cards.length === 0 ? EMPTY_HAND : cards,
    plays: emptyArray(),
    pile: emptyArray(),
    limbo: emptyArray(),
    charged: emptyArray()
  };
}

export function withAction(player: TurboHearts.Player, action: Action, legalPlays?: Card[]): TurboHearts.Player {
  return {
    ...player,
    action,
    legalPlays: legalPlays ?? emptyArray()
  };
}

export function withSentPass(fromPlayer: TurboHearts.Player, passCards: Card[]): TurboHearts.Player {
  const hidden = passCards.length === 0;
  const limbo: Card[] = hidden ? EMPTY_PASS : passCards;
  let hand: Card[];
  if (hidden) {
    hand = [...fromPlayer.hand];
    hand.splice(0, 3);
  } else {
    hand = notCardsOf(fromPlayer.hand, passCards);
  }
  return {
    ...fromPlayer,
    limbo,
    hand
  };
}

export function withReceivePass(
  fromPlayer: TurboHearts.Player,
  toPlayer: TurboHearts.Player,
  passCards: Card[]
): { from: TurboHearts.Player; to: TurboHearts.Player } {
  if (fromPlayer === toPlayer) {
    const actualPassCards = passCards.length === 0 ? EMPTY_PASS : passCards;
    const combined = [...toPlayer.hand, ...actualPassCards];
    sortCards(combined);
    const self: TurboHearts.Player = {
      ...toPlayer,
      limbo: emptyArray(),
      hand: combined
    };
    return {
      from: self,
      to: self
    };
  } else {
    const from: TurboHearts.Player = {
      ...fromPlayer,
      limbo: emptyArray()
    };
    const incoming = passCards.length === 0 ? EMPTY_PASS : passCards;
    const combined = [...toPlayer.hand, ...incoming];
    sortCards(combined);
    const to: TurboHearts.Player = {
      ...toPlayer,
      hand: combined
    };
    return { from, to };
  }
}

export function withCharge(player: TurboHearts.Player, cards: Card[]) {
  if (cards.length === 0) {
    return player;
  }
  const fromHand = cardsOf(player.hand, cards);
  let hand: Card[];
  if (fromHand.length === cards.length) {
    hand = notCardsOf(player.hand, cards);
  } else {
    hand = [...player.hand];
    hand.splice(0, cards.length);
  }
  return {
    ...player,
    charged: cards,
    hand
  };
}

export function withPlay(player: TurboHearts.Player, card: Card) {
  const plays = [...player.plays, card];
  let hand = notCardsOf(player.hand, [card]);
  let charged = player.charged;
  if (hand.length === player.hand.length) {
    charged = notCardsOf(player.charged, [card]);
    if (charged.length === player.charged.length) {
      hand = [...player.hand];
      hand.splice(0, 1);
    }
  }
  return {
    ...player,
    plays,
    hand,
    charged,
    legalPlays: emptyArray()
  };
}

export function withEndTrick(player: TurboHearts.Player, plays: Card[], winner: boolean): TurboHearts.Player {
  const pile = winner ? [...player.pile, ...plays] : player.pile;
  return {
    ...player,
    plays: emptyArray(),
    pile
  };
}

export function emptyStateSnapshot(userId: string): TurboHearts.StateSnapshot {
  return {
    index: 0,
    event: { type: "initial" },
    north: newPlayer("bot", "north"),
    east: newPlayer("bot", "east"),
    south: newPlayer("bot", "south"),
    west: newPlayer("bot", "west"),
    pass: "left",
    userId,
    handNumber: 0,
    trickNumber: 0,
    playNumber: 0
  };
}
