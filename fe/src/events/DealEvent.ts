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
import {
  Card,
  DealEventData,
  Event,
  PointWithRotation,
  SpriteCard
} from "../types";
import { groupCards } from "./groupCards";

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
              card.sprite.texture = this.th.app.loader.resources[
                card.card
              ].texture;
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
    this.th.topCards = this.createSpriteCards(this.event.north, TOP);
    this.th.rightCards = this.createSpriteCards(this.event.east, RIGHT);
    this.th.bottomCards = this.createSpriteCards(this.event.south, BOTTOM);
    this.th.leftCards = this.createSpriteCards(this.event.west, LEFT);
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
