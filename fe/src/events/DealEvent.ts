import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import {
  ANIMATION_DELAY,
  ANIMATION_DURATION,
  BOTTOM,
  CARD_SCALE,
  LEFT,
  RIGHT,
  TABLE_CENTER_X,
  TABLE_CENTER_Y,
  TOP
} from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Card, DealEventData, Event, PointWithRotation, Position, Seat, SpriteCard } from "../types";
import { groupCards } from "./groupCards";

const handAccessors: {
  [bottomSeat: string]: {
    [position: string]: (event: DealEventData) => Card[];
  };
} = {};
handAccessors["north"] = {};
handAccessors["north"]["top"] = event => event.south;
handAccessors["north"]["right"] = event => event.west;
handAccessors["north"]["bottom"] = event => event.north;
handAccessors["north"]["left"] = event => event.east;
handAccessors["east"] = {};
handAccessors["east"]["top"] = event => event.west;
handAccessors["east"]["right"] = event => event.north;
handAccessors["east"]["bottom"] = event => event.east;
handAccessors["east"]["left"] = event => event.south;
handAccessors["south"] = {};
handAccessors["south"]["top"] = event => event.north;
handAccessors["south"]["right"] = event => event.east;
handAccessors["south"]["bottom"] = event => event.south;
handAccessors["south"]["left"] = event => event.west;
handAccessors["west"] = {};
handAccessors["west"]["top"] = event => event.east;
handAccessors["west"]["right"] = event => event.south;
handAccessors["west"]["bottom"] = event => event.west;
handAccessors["west"]["left"] = event => event.north;

function getCardsForPosition(bottomSeat: Seat, position: Position, event: DealEventData) {
  return handAccessors[bottomSeat][position](event);
}

export class DealEvent implements Event {
  private tweens: TWEEN.Tween[] = [];

  constructor(private th: TurboHearts, private event: DealEventData) {
    this.th.pass = event.pass;
  }

  private createSpriteCards(hand: Card[], center: PointWithRotation) {
    const cards: SpriteCard[] = [];
    if (hand.length === 0) {
      for (let i = 0; i < 13; i++) {
        cards.push({
          card: "BACK",
          sprite: new PIXI.Sprite(this.th.app.loader.resources["BACK"].texture),
          hidden: true
        });
      }
    } else {
      for (const card of hand) {
        cards.push({
          card,
          sprite: new PIXI.Sprite(this.th.app.loader.resources["BACK"].texture),
          hidden: true
        });
      }
    }
    for (const card of cards) {
      card.sprite.scale.set(CARD_SCALE);
      card.sprite.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
      card.sprite.anchor.set(0.5, 0.5);
      card.sprite.rotation = -Math.PI;
    }
    let delay = 0;
    const duration = ANIMATION_DURATION;
    const interval = ANIMATION_DELAY;
    const dests = groupCards(cards, center.x, center.y, center.rotation);
    for (let i = 0; i < cards.length; i++) {
      const card = cards[i];
      const dest = dests[i];
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to({ x: dest.x, y: dest.y }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .onComplete(() => {
            if (card.card !== "BACK") {
              card.hidden = false;
              card.sprite.texture = this.th.app.loader.resources[card.card].texture;
            }
          })
          .start()
      );
      this.tweens.push(
        new TWEEN.Tween(card.sprite)
          .to({ rotation: center.rotation }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.th.app.stage.addChild(card.sprite);
      delay += interval;
    }
    return cards;
  }

  public begin() {
    this.th.topPlayer.cards = this.createSpriteCards(getCardsForPosition(this.th.bottomSeat, "top", this.event), TOP);
    this.th.rightPlayer.cards = this.createSpriteCards(
      getCardsForPosition(this.th.bottomSeat, "right", this.event),
      RIGHT
    );
    this.th.bottomPlayer.cards = this.createSpriteCards(
      getCardsForPosition(this.th.bottomSeat, "bottom", this.event),
      BOTTOM
    );
    this.th.leftPlayer.cards = this.createSpriteCards(
      getCardsForPosition(this.th.bottomSeat, "left", this.event),
      LEFT
    );
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
