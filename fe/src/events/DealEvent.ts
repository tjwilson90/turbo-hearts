import { TurboHearts } from "../game/TurboHearts";
import { DealEventData, Card, SpriteCard, Event, Point } from "../types";
import * as PIXI from "pixi.js";
import TWEEN from "@tweenjs/tween.js";

const SIZE = 1000;
const INSET = 40;
const CARDS_LENGTH = 400;

export class DealEvent implements Event {
  private tweens: TWEEN.Tween[] = [];

  constructor(private th: TurboHearts, private event: DealEventData) {
    this.th.pass = event.pass;
  }

  private createSpriteCards(
    hand: Card[],
    from: Point,
    to: Point,
    rotation: number
  ) {
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
    let delay = 0;
    let i = 0;
    const duration = 300;
    const interval = 80;
    for (const card of cards) {
      card.sprite.scale.set(0.5);
      card.sprite.position.set(500, 500);
      card.sprite.anchor.set(0.5, 0.5);
      card.sprite.rotation = rotation;
      const destX = from.x + (to.x - from.x) * (i / 12);
      const destY = from.y + (to.y - from.y) * (i / 12);
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to({ x: destX, y: destY }, duration)
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
          // .to({ rotation: rotation + Math.PI * 2 }, duration)
          .to({ rotation: rotation }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.th.app.stage.addChild(card.sprite);
      delay += interval;
      i++;
    }
    return cards;
  }

  public begin() {
    const d = (SIZE - CARDS_LENGTH) / 2;
    this.th.topCards = this.createSpriteCards(
      this.event.north,
      { x: SIZE - d, y: INSET },
      { x: d, y: INSET },
      Math.PI
    );
    this.th.rightCards = this.createSpriteCards(
      this.event.east,
      { x: SIZE - INSET, y: SIZE - d },
      { x: SIZE - INSET, y: d },
      Math.PI / 2
    );
    this.th.bottomCards = this.createSpriteCards(
      this.event.south,
      { x: d, y: SIZE - INSET },
      { x: SIZE - d, y: SIZE - INSET },
      0
    );
    this.th.leftCards = this.createSpriteCards(
      this.event.west,
      { x: INSET, y: d },
      { x: INSET, y: SIZE - d },
      (Math.PI * 3) / 2
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
