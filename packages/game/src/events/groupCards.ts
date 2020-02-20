import { CARD_OVERLAP } from "../const";
import { SpriteCard } from "../types";

/**
 * @param cards The cards to group
 * @param x the center x point for the group
 * @param y the center y point for the group
 * @param offset how much of an underlapping card to show
 * @param rotation
 */
export function groupCards(cards: SpriteCard[], x: number, y: number, rotation: number, overlap = CARD_OVERLAP) {
  const cosR = Math.cos(rotation);
  const sinR = Math.sin(rotation);
  const dx = cosR * overlap;
  const dy = sinR * overlap;
  const result = [];
  const totalLength = (cards.length - 1) * overlap;
  const offsetX = (cosR * totalLength) / 2;
  const offsetY = (sinR * totalLength) / 2;
  for (let i = 0; i < cards.length; i++) {
    result.push({ x: x + i * dx - offsetX, y: y + i * dy - offsetY });
  }
  return result;
}
