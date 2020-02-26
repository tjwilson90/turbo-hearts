import { Card, EventData, Pass } from "../types";
import { emptyArray } from "../util/array";
import { sortSpriteCards, sortCards } from "./sortCards";

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

export namespace TurboHearts {
  export interface Player {
    type: "bot" | "human";
    name: string;
    hand: Card[];
    plays: Card[];
    pile: Card[];
    limbo: Card[];
    charged: Card[];
    legalPlays: Card[];
    toPlay: boolean;
  }

  export interface StateSnapshot {
    event: EventData;
    north: Player;
    east: Player;
    south: Player;
    west: Player;

    pass: Pass;
    userName: string;
    handNumber: number;
    trickNumber: number;
    playNumber: number;
  }

  export interface Game {
    snapshots: StateSnapshot[];
  }
}

export function newPlayer(type: "bot" | "human", name: string): TurboHearts.Player {
  return {
    type,
    name,
    hand: emptyArray(),
    plays: emptyArray(),
    pile: emptyArray(),
    limbo: emptyArray(),
    charged: emptyArray(),
    legalPlays: emptyArray(),
    toPlay: false
  };
}

export function withDeal(player: TurboHearts.Player, cards: Card[]): TurboHearts.Player {
  return {
    ...player,
    hand: cards.length === 0 ? EMPTY_HAND : cards,
    plays: emptyArray(),
    pile: emptyArray(),
    limbo: emptyArray(),
    charged: emptyArray()
  };
}

export function withToPlay(player: TurboHearts.Player, toPlay: boolean, legalPlays?: Card[]): TurboHearts.Player {
  return {
    ...player,
    toPlay,
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
    const combined = [...toPlayer.hand, ...passCards];
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
  if (hand.length === player.hand.length) {
    hand = [...player.hand];
    hand.splice(0, 1);
  }
  return {
    ...player,
    plays,
    hand,
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

export function emptyStateSnapshot(userName: string): TurboHearts.StateSnapshot {
  return {
    event: { type: "initial" },
    north: newPlayer("bot", "north"),
    east: newPlayer("bot", "east"),
    south: newPlayer("bot", "south"),
    west: newPlayer("bot", "west"),
    pass: "left",
    userName,
    handNumber: 0,
    trickNumber: 0,
    playNumber: 0
  };
}
