import { SpriteCard, Card } from "../types";

export function spriteCardsOf(spriteCards: SpriteCard[], rawCards: Card[]) {
  const set = new Set(rawCards);
  return spriteCards.filter(c => set.has(c.card));
}

export function spriteCardsOfNot(spriteCards: SpriteCard[], rawCards: Card[]) {
  const set = new Set(rawCards);
  return spriteCards.filter(c => !set.has(c.card));
}

export function sleep(millis: number): Promise<void> {
  return new Promise(resolve => {
    setTimeout(resolve, millis);
  });
}
