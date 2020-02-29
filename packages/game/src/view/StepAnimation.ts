import TWEEN from "@tweenjs/tween.js";
import {
  BOTTOM,
  CARD_DROP_SHADOW,
  CARD_OVERLAP,
  CHARGE_OVERLAP,
  FAST_ANIMATION_DELAY,
  FAST_ANIMATION_DURATION,
  LEFT,
  RIGHT,
  TOP,
  TRICK_COLLECTION_PAUSE,
  Z_CHARGED_CARDS,
  Z_DEALING_CARDS,
  Z_PILE_CARDS,
  Z_PLAYED_CARDS,
  Z_TRANSIT_CARDS,
  Z_HAND_CARDS,
  TABLE_CENTER_X,
  TABLE_CENTER_Y
} from "../const";
import { groupCards } from "../events/groupCards";
import { sleep, spriteCardsOf } from "../events/helpers";
import { sortCards, sortSpriteCards } from "../game/sortCards";
import { TurboHearts } from "../game/stateSnapshot";
import {
  Animation,
  Card,
  ChargeEventData,
  PlayerCardPositions,
  PlayerSpriteCards,
  Point,
  Position,
  ReceivePassEventData,
  Seat,
  SpriteCard,
  SendPassEventData
} from "../types";
import { pushAll, removeAll } from "../util/array";
import { PASS_POSITION_OFFSETS, addToSeat } from "../game/snapshotter";
import { LIMBO_POSITIONS_FOR_BOTTOM_SEAT } from "./TurboHeartsStage";

const TRUE_SEAT_ORDER_FOR_BOTTOM_SEAT: { [bottomSeat in Seat]: Seat[] } = {
  // [top, right, bottom, left]
  north: ["south", "west", "north", "east"],
  east: ["west", "north", "east", "south"],
  south: ["north", "east", "south", "west"],
  west: ["east", "south", "west", "north"]
};

const POSITIONS: { [bottomSeat in Seat]: { [trueSeat in Seat]: Position } } = {
  north: {
    north: "bottom",
    east: "left",
    south: "top",
    west: "right"
  },
  east: {
    north: "right",
    east: "bottom",
    south: "left",
    west: "top"
  },
  south: {
    north: "top",
    east: "right",
    south: "bottom",
    west: "left"
  },
  west: {
    north: "left",
    east: "top",
    south: "right",
    west: "bottom"
  }
};
const POSITION_ORDER: Position[] = ["top", "right", "bottom", "left"];
const POSITION_LAYOUTS = { top: TOP, right: RIGHT, bottom: BOTTOM, left: LEFT };

export class StepAnimation implements Animation {
  private finished = false;

  constructor(
    private cardTextures: PIXI.IResourceDictionary,
    private cardCreator: (card: Card, hidden: boolean) => SpriteCard,
    private zSort: () => void,
    private bottomSeat: Seat,
    private previous: TurboHearts.StateSnapshot,
    private next: TurboHearts.StateSnapshot,
    private top: PlayerSpriteCards,
    private right: PlayerSpriteCards,
    private bottom: PlayerSpriteCards,
    private left: PlayerSpriteCards
  ) {}

  public start() {
    switch (this.next.event.type) {
      case "deal":
        this.animateDeal();
        break;
      case "send_pass":
        this.animateSendPass(this.next.event);
        break;
      case "recv_pass":
        this.animateReceivePass(this.next.event);
        break;
      case "charge":
        this.animateCharge(this.next.event);
        break;
      case "play":
        this.animatePlay();
        break;
      case "end_trick":
        this.animateEndTrick(this.next.event.winner);
        break;
      default:
        this.finished = true;
        return;
    }
  }

  public isFinished() {
    return this.finished;
  }

  private animateDeal() {
    const backTexture = this.cardTextures["BACK"].texture;
    const seatOrder = TRUE_SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
    const deckCards: SpriteCard[] = [];
    for (let i = 0; i < 4; i++) {
      const spriteCards = this[POSITION_ORDER[i]];
      pushAll(deckCards, spriteCards.pile);
    }
    if (deckCards.length === 0) {
      for (let i = 0; i < 52; i++) {
        deckCards.push(this.cardCreator("BACK", false));
      }
    } else if (deckCards.length === 52) {
      for (let i = 0; i < 52; i++) {
        deckCards[i].hidden = false;
        deckCards[i].card = "BACK";
      }
    } else {
      throw new Error("illegal deck");
    }
    for (let i = 0; i < 4; i++) {
      const player = this.next[seatOrder[i]];
      sortCards(player.hand);
      const spriteCards = this[POSITION_ORDER[i]];
      for (let j = 0; j < 13; j++) {
        const spriteCard = deckCards.pop()!;
        spriteCard.card = player.hand[j];
        spriteCard.hidden = false;
        spriteCard.sprite.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
        spriteCard.sprite.texture = backTexture;
        spriteCard.sprite.rotation = -Math.PI;
        spriteCard.sprite.filters = [];
        spriteCards.hand.push(spriteCard);
      }
    }
    let delay = 0;
    let finished = 0;
    let started = 0;
    const animateDealCard = (card: SpriteCard, dest: Point, rotation: number) => {
      card.sprite.zIndex = Z_DEALING_CARDS - started;

      new TWEEN.Tween(card.sprite.position)
        .to(dest, FAST_ANIMATION_DURATION)
        .easing(TWEEN.Easing.Quadratic.Out)
        .delay(delay)
        .onStart(() => {
          card.sprite.filters = [CARD_DROP_SHADOW];
        })
        .onComplete(() => {
          if (card.hidden && card.sprite.texture !== backTexture) {
            card.sprite.texture = backTexture;
          }
          if (!card.hidden && card.sprite.texture === backTexture) {
            card.sprite.texture = this.cardTextures[card.card].texture;
          }
          finished++;
          if (finished === started) {
            this.finished = true;
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
    };
    const topDests = groupCards(this.top.hand, TOP.x, TOP.y, TOP.rotation);
    const rightDests = groupCards(this.right.hand, RIGHT.x, RIGHT.y, RIGHT.rotation);
    const bottomDests = groupCards(this.bottom.hand, BOTTOM.x, BOTTOM.y, BOTTOM.rotation);
    const leftDests = groupCards(this.left.hand, LEFT.x, LEFT.y, LEFT.rotation);
    for (let i = 12; i >= 0; i--) {
      animateDealCard(this.top.hand[i], topDests[i], TOP.rotation);
      animateDealCard(this.right.hand[i], rightDests[i], RIGHT.rotation);
      animateDealCard(this.bottom.hand[i], bottomDests[i], BOTTOM.rotation);
      animateDealCard(this.left.hand[i], leftDests[i], LEFT.rotation);
    }
    this.zSort();
  }

  private getSpritePlayer(trueSeat: Seat) {
    const position = POSITIONS[this.bottomSeat][trueSeat];
    return {
      sprites: this[position],
      layout: POSITION_LAYOUTS[position]
    };
  }

  private async animateSendPass(event: SendPassEventData) {
    const fromPlayer = this.getSpritePlayer(event.from);
    let cardsToMove: SpriteCard[];
    if (event.cards.length === 0) {
      cardsToMove = fromPlayer.sprites.hand.splice(0, 3);
      pushAll(fromPlayer.sprites.limbo, cardsToMove);
    } else {
      cardsToMove = spriteCardsOf(fromPlayer.sprites.hand, event.cards);
      removeAll(fromPlayer.sprites.hand, cardsToMove);
      pushAll(fromPlayer.sprites.limbo, cardsToMove);
      for (const card of cardsToMove) {
        card.sprite.texture = this.cardTextures["BACK"].texture;
      }
    }
    const layout = LIMBO_POSITIONS_FOR_BOTTOM_SEAT[this.bottomSeat][event.from][this.next.pass];
    await Promise.all([
      this.animateCards(cardsToMove, layout.x, layout.y, layout.rotation),
      this.animateCards(fromPlayer.sprites.hand, fromPlayer.layout.x, fromPlayer.layout.y, fromPlayer.layout.rotation)
    ]);
    this.finished = true;
  }

  private async animateReceivePass(event: ReceivePassEventData) {
    const fromSeat = addToSeat(event.to, -PASS_POSITION_OFFSETS[this.next.pass]);
    const toPlayer = this.getSpritePlayer(event.to);
    const fromPlayer = this.getSpritePlayer(fromSeat);
    const received = [...event.cards];
    while (fromPlayer.sprites.limbo.length > 0) {
      // Note: this is mutating both hand and limbo arrays
      const fromLimbo = fromPlayer.sprites.limbo.pop()!;
      if (fromLimbo.card === "BACK" && received.length > 0) {
        fromLimbo.card = received.pop()!;
        fromLimbo.sprite.texture = this.cardTextures[fromLimbo.card].texture;
      } else if (fromLimbo.card !== "BACK" && received.length === 0) {
        // Passing known cards into another hand
        fromLimbo.card = "BACK";
        fromLimbo.sprite.texture = this.cardTextures["BACK"].texture;
      } else {
        // TODO: receiving own cards back, reveal
      }
      toPlayer.sprites.hand.push(fromLimbo);
    }
    sortSpriteCards(toPlayer.sprites.hand);
    let i = Z_HAND_CARDS;
    for (const card of toPlayer.sprites.hand) {
      card.sprite.zIndex = i++;
    }
    this.zSort();
    await this.animateCards(toPlayer.sprites.hand, toPlayer.layout.x, toPlayer.layout.y, toPlayer.layout.rotation);
    this.finished = true;
  }

  private async animateCharge(event: ChargeEventData) {
    if (event.cards.length === 0) {
      this.finished = true;
      return;
    }
    const player = this.getSpritePlayer(event.seat);

    const chargeCards = spriteCardsOf(player.sprites.hand, event.cards);
    if (chargeCards.length !== event.cards.length) {
      // Pull from unknown hand
      const charged = player.sprites.hand.splice(0, event.cards.length);
      for (let i = 0; i < charged.length; i++) {
        charged[i].card = event.cards[i];
      }
      pushAll(player.sprites.charged, charged);
    } else {
      // Pull from known hand
      removeAll(player.sprites.hand, chargeCards);
      pushAll(player.sprites.charged, chargeCards);
    }
    await Promise.all([
      this.animateCards(
        player.sprites.charged,
        player.layout.chargeX,
        player.layout.chargeY,
        player.layout.rotation,
        CHARGE_OVERLAP
      ),
      this.animateCards(player.sprites.hand, player.layout.x, player.layout.y, player.layout.rotation)
    ]);
    for (const card of player.sprites.charged) {
      card.sprite.zIndex = Z_CHARGED_CARDS;
    }
    this.zSort();
    this.finished = true;
  }

  private async animatePlay() {
    const seatOrder = TRUE_SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
    for (let i = 0; i < 4; i++) {
      const playerPrev = this.previous[seatOrder[i]];
      const playerNext = this.next[seatOrder[i]];
      if (playerNext.plays.length !== playerPrev.plays.length) {
        const spriteCards = this[POSITION_ORDER[i]];
        const layout = POSITION_LAYOUTS[POSITION_ORDER[i]];
        const playedCard = playerNext.plays[playerNext.plays.length - 1];
        let toRemove = spriteCardsOf([...spriteCards.hand, ...spriteCards.charged], [playedCard]);
        let card;
        if (toRemove.length !== 1) {
          // remove from hidden hand
          toRemove = spriteCards.hand.splice(0, 1);
          card = toRemove[0];
          card.card = playedCard;
          card.sprite.texture = this.cardTextures[playedCard].texture;
          card.sprite.zIndex = Z_TRANSIT_CARDS;
        } else {
          card = toRemove[0];
        }
        removeAll(spriteCards.hand, toRemove);
        removeAll(spriteCards.charged, toRemove);
        pushAll(spriteCards.plays, toRemove);

        this.zSort();
        await Promise.all([
          this.animateCards(spriteCards.plays, layout.playX, layout.playY, layout.rotation, CARD_OVERLAP, true),
          this.animateCards(spriteCards.hand, layout.x, layout.y, layout.rotation, CARD_OVERLAP)
        ]);
        this.finished = true;
        card.sprite.zIndex = Z_PLAYED_CARDS + this.next.playNumber;
        this.zSort();
        break;
      }
    }
  }

  private async animateEndTrick(winner: Seat) {
    await sleep(TRICK_COLLECTION_PAUSE);
    const seatOrder = TRUE_SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
    const pileCards: SpriteCard[] = [];
    let winnerPlayer: PlayerSpriteCards;
    let winnerLayout: PlayerCardPositions;
    for (let i = 0; i < 4; i++) {
      const seat = seatOrder[i];
      const position = POSITION_ORDER[i];
      const spritePlayer = this[position];
      if (seat === winner) {
        winnerPlayer = spritePlayer;
        winnerLayout = POSITION_LAYOUTS[position];
      }
      pushAll(pileCards, spritePlayer.plays);
      spritePlayer.plays = [];
    }
    let first = true;
    for (const card of pileCards) {
      card.sprite.zIndex = Z_PILE_CARDS;
      card.sprite.texture = this.cardTextures["BACK"].texture;
      if (!first) {
        card.sprite.filters = [];
      }
      first = false;
    }
    this.zSort();
    pushAll(winnerPlayer!.pile, pileCards);
    await this.animateCards(
      winnerPlayer!.pile,
      winnerLayout!.pileX,
      winnerLayout!.pileY,
      winnerLayout!.pileRotation,
      0
    );
    this.finished = true;
  }

  private animateCards(
    cards: SpriteCard[],
    x: number,
    y: number,
    rotation: number,
    overlap = CARD_OVERLAP,
    invert = false
  ) {
    const cardDests = groupCards(cards, x, y, rotation, overlap, invert);
    return new Promise(resolve => {
      let finished = 0;
      let started = 0;
      let i = 0;
      if (cards.length === 0) {
        resolve();
      }
      const backTexture = this.cardTextures["BACK"].texture;
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
}
