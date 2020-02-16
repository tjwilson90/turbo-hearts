import TWEEN from "@tweenjs/tween.js";
import { FAST_ANIMATION_DURATION } from "../../const";
import { TurboHearts } from "../../game/TurboHearts";
import { Seat, SpriteCard } from "../../types";
import { groupCards } from "../groupCards";
import { getHandPosition } from "../handPositions";
import { getPlayerAccessor } from "../playerAccessors";

export function animateCards(cards: SpriteCard[], x: number, y: number, rotation: number) {
  const cardDests = groupCards(cards, x, y, rotation);
  return new Promise(resolve => {
    let finished = 0;
    let started = 0;
    let i = 0;
    if (cards.length === 0) {
      resolve();
    }
    for (const card of cards) {
      new TWEEN.Tween(card.sprite.position)
        .to(cardDests[i], FAST_ANIMATION_DURATION)
        .easing(TWEEN.Easing.Quadratic.Out)
        .onComplete(() => {
          finished++;
          if (finished === started) {
            resolve();
          }
        })
        .start();
      started++;
      new TWEEN.Tween(card.sprite)
        .to({ rotation }, FAST_ANIMATION_DURATION)
        .easing(TWEEN.Easing.Quadratic.Out)
        .onComplete(() => {
          finished++;
          if (finished === started) {
            resolve();
          }
        })
        .start();
      started++;
      i++;
    }
  });
}

export function animateHand(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(player.cards, handPosition.x, handPosition.y, handPosition.rotation);
}

export function animatePlay(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(player.playCards, handPosition.playX, handPosition.playY, handPosition.rotation);
}

export function animateCharges(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(player.chargedCards, handPosition.chargeX, handPosition.chargeY, handPosition.rotation);
}
