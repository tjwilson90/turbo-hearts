import TWEEN from "@tweenjs/tween.js";
import {
  BOTTOM,
  CARD_OVERLAP,
  CHARGE_OVERLAP,
  FAST_ANIMATION_DELAY,
  FAST_ANIMATION_DURATION,
  LEFT,
  RIGHT,
  TOP,
  Z_DEALING_CARDS
} from "../../const";
import { TurboHearts } from "../../game/TurboHearts";
import { Point, Seat, SpriteCard } from "../../types";
import { groupCards } from "../groupCards";
import { getHandPosition } from "../handPositions";
import { getPlayerAccessor } from "../playerAccessors";

export function moveCards(
  th: TurboHearts,
  cards: SpriteCard[],
  x: number,
  y: number,
  rotation: number,
  overlap = CARD_OVERLAP
) {
  const cardDests = groupCards(cards, x, y, rotation, overlap);
  let i = 0;
  const backTexture = th.app.loader.resources["BACK"].texture;
  for (const card of cards) {
    if (card.hidden && card.sprite.texture !== backTexture) {
      card.sprite.texture = backTexture;
    }
    if (!card.hidden && card.sprite.texture === backTexture) {
      card.sprite.texture = th.app.loader.resources[card.card].texture;
    }
    card.sprite.position.set(cardDests[i].x, cardDests[i].y);
    card.sprite.rotation = rotation;
    i++;
  }
}

export function animateCards(
  th: TurboHearts,
  cards: SpriteCard[],
  x: number,
  y: number,
  rotation: number,
  overlap = CARD_OVERLAP
) {
  const cardDests = groupCards(cards, x, y, rotation, overlap);
  return new Promise(resolve => {
    let finished = 0;
    let started = 0;
    let i = 0;
    if (cards.length === 0) {
      resolve();
    }
    const backTexture = th.app.loader.resources["BACK"].texture;
    for (const card of cards) {
      new TWEEN.Tween(card.sprite.position)
        .to(cardDests[i], FAST_ANIMATION_DURATION)
        .easing(TWEEN.Easing.Quadratic.Out)
        .onComplete(() => {
          if (card.hidden && card.sprite.texture !== backTexture) {
            card.sprite.texture = backTexture;
          }
          if (!card.hidden && card.sprite.texture === backTexture) {
            card.sprite.texture = th.app.loader.resources[card.card].texture;
          }
          finished++;
          if (finished === started) {
            resolve();
          }
        })
        .start();
      started++;
      const totalRotation = rotation - card.sprite.rotation;
      // Prevent overly spinny cards
      if (Math.abs(totalRotation) > Math.PI) {
        card.sprite.rotation += 2 * Math.PI * Math.sign(totalRotation);
      }
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

export function moveDeal(th: TurboHearts) {
  const topDests = groupCards(th.topPlayer.cards, TOP.x, TOP.y, TOP.rotation);
  const rightDests = groupCards(th.rightPlayer.cards, RIGHT.x, RIGHT.y, RIGHT.rotation);
  const bottomDests = groupCards(th.bottomPlayer.cards, BOTTOM.x, BOTTOM.y, BOTTOM.rotation);
  const leftDests = groupCards(th.leftPlayer.cards, LEFT.x, LEFT.y, LEFT.rotation);
  const backTexture = th.app.loader.resources["BACK"].texture;
  let zIndex = Z_DEALING_CARDS;
  function moveCard(card: SpriteCard, dest: Point, rotation: number) {
    card.sprite.zIndex = zIndex--;
    if (card.hidden && card.sprite.texture !== backTexture) {
      card.sprite.texture = backTexture;
    }
    if (!card.hidden && card.sprite.texture === backTexture) {
      card.sprite.texture = th.app.loader.resources[card.card].texture;
    }
    card.sprite.position.set(dest.x, dest.y);
    card.sprite.rotation = rotation;
    return;
  }
  for (let i = 12; i >= 0; i--) {
    moveCard(th.topPlayer.cards[i], topDests[i], TOP.rotation);
    moveCard(th.rightPlayer.cards[i], rightDests[i], RIGHT.rotation);
    moveCard(th.bottomPlayer.cards[i], bottomDests[i], BOTTOM.rotation);
    moveCard(th.leftPlayer.cards[i], leftDests[i], LEFT.rotation);
  }
  th.app.stage.sortChildren();
}

export function animateDeal(th: TurboHearts) {
  const topDests = groupCards(th.topPlayer.cards, TOP.x, TOP.y, TOP.rotation);
  const rightDests = groupCards(th.rightPlayer.cards, RIGHT.x, RIGHT.y, RIGHT.rotation);
  const bottomDests = groupCards(th.bottomPlayer.cards, BOTTOM.x, BOTTOM.y, BOTTOM.rotation);
  const leftDests = groupCards(th.leftPlayer.cards, LEFT.x, LEFT.y, LEFT.rotation);
  const backTexture = th.app.loader.resources["BACK"].texture;
  let delay = 0;
  let finished = 0;
  let started = 0;
  let zIndex = Z_DEALING_CARDS;
  function animateDealCard(card: SpriteCard, dest: Point, rotation: number, resolve: () => void) {
    card.sprite.zIndex = zIndex--;

    new TWEEN.Tween(card.sprite.position)
      .to(dest, FAST_ANIMATION_DURATION)
      .easing(TWEEN.Easing.Quadratic.Out)
      .delay(delay)
      .onComplete(() => {
        if (card.hidden && card.sprite.texture !== backTexture) {
          card.sprite.texture = backTexture;
        }
        if (!card.hidden && card.sprite.texture === backTexture) {
          card.sprite.texture = th.app.loader.resources[card.card].texture;
        }
        finished++;
        if (finished === started) {
          resolve();
        }
      })
      .start();
    new TWEEN.Tween(card.sprite)
      .to({ rotation }, FAST_ANIMATION_DURATION)
      .delay(delay)
      .easing(TWEEN.Easing.Quadratic.Out)
      .start();
    started++;
    delay += FAST_ANIMATION_DELAY;
  }
  return new Promise(resolve => {
    for (let i = 12; i >= 0; i--) {
      animateDealCard(th.topPlayer.cards[i], topDests[i], TOP.rotation, resolve);
      animateDealCard(th.rightPlayer.cards[i], rightDests[i], RIGHT.rotation, resolve);
      animateDealCard(th.bottomPlayer.cards[i], bottomDests[i], BOTTOM.rotation, resolve);
      animateDealCard(th.leftPlayer.cards[i], leftDests[i], LEFT.rotation, resolve);
    }
    th.app.stage.sortChildren();
  });
}

export function moveHand(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  moveCards(th, player.cards, handPosition.x, handPosition.y, handPosition.rotation);
}

export function animateHand(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(th, player.cards, handPosition.x, handPosition.y, handPosition.rotation);
}

export function movePlay(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  moveCards(th, player.playCards, handPosition.playX, handPosition.playY, handPosition.rotation);
}

export function animatePlay(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(th, player.playCards, handPosition.playX, handPosition.playY, handPosition.rotation);
}

export function moveCharges(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return moveCards(
    th,
    player.chargedCards,
    handPosition.chargeX,
    handPosition.chargeY,
    handPosition.rotation,
    CHARGE_OVERLAP
  );
}

export function animateCharges(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(
    th,
    player.chargedCards,
    handPosition.chargeX,
    handPosition.chargeY,
    handPosition.rotation,
    CHARGE_OVERLAP
  );
}

export function movePile(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return moveCards(th, player.pileCards, handPosition.pileX, handPosition.pileY, handPosition.pileRotation, 0);
}

export function animatePile(th: TurboHearts, seat: Seat) {
  const player = getPlayerAccessor(th.bottomSeat, seat)(th);
  const handPosition = getHandPosition(th.bottomSeat, seat);
  return animateCards(th, player.pileCards, handPosition.pileX, handPosition.pileY, handPosition.pileRotation, 0);
}
