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
  TABLE_CENTER_X,
  TABLE_CENTER_Y,
  TOP,
  TRICK_COLLECTION_PAUSE,
  Z_CHARGED_CARDS,
  Z_DEALING_CARDS,
  Z_HAND_CARDS,
  Z_LIMBO_CARDS,
  Z_PILE_CARDS,
  Z_PLAYED_CARDS,
  Z_TRANSIT_CARDS,
  CLAIM_PAUSE,
  TABLE_SIZE
} from "../const";
import { sortCards, sortSpriteCards } from "../game/sortCards";
import { TurboHearts } from "../game/stateSnapshot";
import {
  Animation,
  Card,
  ChargeEventData,
  PlayerCardPositions,
  PlayerSpriteCards,
  Point,
  Seat,
  SpriteCard,
  ClaimEventData
} from "../types";
import { pushAll, removeAll } from "../util/array";
import { groupCards } from "../util/groupCards";
import { sleep, spriteCardsOf } from "../util/helpers";
import { POSITIONS, POSITION_ORDER, SEAT_ORDER_FOR_BOTTOM_SEAT, addToSeat, PASS_OFFSETS } from "../util/seatPositions";
import { LIMBO_POSITIONS_FOR_BOTTOM_SEAT } from "./TurboHeartsStage";
import { EMPTY_HAND } from "../game/stateSnapshot";

const POSITION_LAYOUTS = { top: TOP, right: RIGHT, bottom: BOTTOM, left: LEFT };

export class StepAnimation implements Animation {
  private finished = false;

  constructor(
    private cardTextures: PIXI.IResourceDictionary,
    private cardCreator: (card: Card, hidden: boolean) => SpriteCard,
    private zSort: () => void,
    private spectatorMode: boolean,
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
        this.animateSendPass(this.next.event.from, this.next.event.cards, -1);
        break;
      case "recv_pass":
        this.animateReceivePass(this.next.event.to, this.next.event.cards, -1);
        break;
      case "hidden_send_pass":
        this.animateSendPass(this.next.event.from, [], this.next.event.count);
        break;
      case "hidden_recv_pass":
        this.animateReceivePass(this.next.event.to, [], this.next.event.count);
        break;
      case "charge":
        this.animateCharge(this.next.event);
        break;
      case "play":
        this.animatePlay();
        break;
      case "claim":
        this.animateClaim(this.next.event);
        break;
      case "end_trick":
        this.animateEndTrick(this.next.event.winner);
        break;
      case "game_complete":
        this.animateGameComplete();
        break;
      default:
        this.finished = true;
        return;
    }
  }

  public isFinished() {
    return this.finished;
  }

  private collectDeckCards() {
    const deckCards: SpriteCard[] = [];
    for (let i = 0; i < 4; i++) {
      const spriteCards = this[POSITION_ORDER[i]];
      // Clear all areas that could contain cards, as claims can be accepted mid-game and mid-play.
      pushAll(deckCards, spriteCards.pile);
      pushAll(deckCards, spriteCards.hand);
      pushAll(deckCards, spriteCards.charged);
      pushAll(deckCards, spriteCards.plays);
      pushAll(deckCards, spriteCards.limbo);
      spriteCards.pile = [];
      spriteCards.hand = [];
      spriteCards.charged = [];
      spriteCards.plays = [];
      spriteCards.limbo = [];
    }
    return deckCards;
  }

  private animateDeal() {
    const backTexture = this.cardTextures["BACK"].texture;
    const seatOrder = SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
    const deckCards: SpriteCard[] = this.collectDeckCards();
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
          card.sprite.zIndex = Z_HAND_CARDS - finished;
          finished++;
          if (finished === 52) {
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

  private async animateSendPass(from: Seat, cards: Card[], count: number) {
    const fromPlayer = this.getSpritePlayer(from);
    let cardsToMove: SpriteCard[];
    if (count >= 0) {
      cardsToMove = fromPlayer.sprites.hand.splice(0, count);
      pushAll(fromPlayer.sprites.limbo, cardsToMove);
    } else {
      cardsToMove = spriteCardsOf(fromPlayer.sprites.hand, cards);
      removeAll(fromPlayer.sprites.hand, cardsToMove);
      pushAll(fromPlayer.sprites.limbo, cardsToMove);
      if (!this.spectatorMode) {
        for (const card of cardsToMove) {
          card.sprite.texture = this.cardTextures["BACK"].texture;
        }
      }
    }
    let i = Z_LIMBO_CARDS;
    for (const card of cardsToMove) {
      card.sprite.zIndex = i++;
    }
    this.zSort();
    const layout = LIMBO_POSITIONS_FOR_BOTTOM_SEAT[this.bottomSeat][from][this.next.pass];
    await Promise.all([
      this.animateCards(cardsToMove, layout.x, layout.y, layout.rotation),
      this.animateCards(fromPlayer.sprites.hand, fromPlayer.layout.x, fromPlayer.layout.y, fromPlayer.layout.rotation)
    ]);
    this.finished = true;
  }

  private async animateReceivePass(to: Seat, cards: Card[], count: number) {
    const fromSeat = addToSeat(to, -PASS_OFFSETS[this.next.pass]);
    const toPlayer = this.getSpritePlayer(to);
    const fromPlayer = this.getSpritePlayer(fromSeat);
    const received = count >= 0 ? EMPTY_HAND.slice(0, count) : [...cards];
    while (fromPlayer.sprites.limbo.length > 0) {
      // Note: this is mutating both hand and limbo arrays
      const fromLimbo = fromPlayer.sprites.limbo.pop()!;
      if (received.length > 0) {
        // Receiving
        fromLimbo.card = received.pop()!;
        fromLimbo.sprite.texture = this.cardTextures[fromLimbo.card].texture;
      } else {
        // Passing known cards into another hand
        fromLimbo.card = "BACK";
        fromLimbo.sprite.texture = this.cardTextures["BACK"].texture;
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
        charged[i].sprite.texture = this.cardTextures[event.cards[i]].texture;
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
    const seatOrder = SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
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

  private async animateClaim(event: ClaimEventData) {
    if (event.seat === this.bottomSeat) {
      this.finished = true;
      return;
    }

    const player = this.getSpritePlayer(event.seat);
    const cardSet = new Set(event.hand);
    for (const spriteCard of player.sprites.charged) {
      cardSet.delete(spriteCard.card);
    }
    if (cardSet.size !== player.sprites.hand.length) {
      throw new Error("illegal hand size for claim");
    }
    const cards = Array.from(cardSet.values());
    sortCards(cards);
    for (let i = 0; i < cards.length; i++) {
      const spriteCard = player.sprites.hand[i];
      const card = cards[i];
      spriteCard.card = card;
      spriteCard.hidden = false;
      spriteCard.sprite.texture = this.cardTextures[card].texture;
      // Do it with style
      await sleep(CLAIM_PAUSE);
    }
    this.finished = true;
  }

  private async animateEndTrick(winner: Seat) {
    await sleep(TRICK_COLLECTION_PAUSE);
    const seatOrder = SEAT_ORDER_FOR_BOTTOM_SEAT[this.bottomSeat];
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

  private async animateGameComplete() {
    const deckCards: SpriteCard[] = this.collectDeckCards();
    const dests = [];
    if (deckCards.length === 52) {
      const spread = TABLE_SIZE / 2;
      for (let i = 0; i < 52; i++) {
        deckCards[i].hidden = false;
        deckCards[i].card = "BACK";
        deckCards[i].sprite.texture = this.cardTextures["BACK"].texture;
        const x = Math.random() * spread - spread / 2 + TABLE_CENTER_X;
        const y = Math.random() * spread - spread / 2 + TABLE_CENTER_Y;
        dests.push({ x, y, rotation: -Math.PI + Math.random() * 2 * Math.PI });
      }
    } else {
      throw new Error("illegal deck");
    }
    let delay = 0;
    let started = 0;
    let finished = 0;
    const animateDeckCard = (card: SpriteCard, dest: Point & { rotation: number }) => {
      card.sprite.zIndex = Z_DEALING_CARDS - started;

      new TWEEN.Tween(card.sprite.position)
        .to(dest, FAST_ANIMATION_DURATION)
        .easing(TWEEN.Easing.Quadratic.Out)
        .delay(delay)
        .onStart(() => {
          card.sprite.filters = [CARD_DROP_SHADOW];
        })
        .onComplete(() => {
          finished++;
          if (finished === 52) {
            this.finished = true;
          }
        })
        .start();
      new TWEEN.Tween(card.sprite)
        .to({ rotation: dest.rotation }, FAST_ANIMATION_DURATION)
        .delay(delay)
        .easing(TWEEN.Easing.Quadratic.Out)
        .start();
      started++;
      delay += FAST_ANIMATION_DELAY;
    };

    for (let i = 0; i < 52; i++) {
      animateDeckCard(deckCards[i], dests[i]);
    }
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
            if (finished === 2 * cards.length) {
              resolve();
            }
          })
          .start();
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
            if (finished === 2 * cards.length) {
              resolve();
            }
          })
          .start();
        i++;
      }
    });
  }
}
