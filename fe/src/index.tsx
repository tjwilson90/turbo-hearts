import "./styles/style.scss";
import { TEST_EVENTS } from "./test";
import * as PIXI from "pixi.js";
import TWEEN, { Tween } from "@tweenjs/tween.js";
import {
  CARDS,
  Card,
  DealEventData,
  SendPassData,
  Pass,
  EventData
} from "./types";

const SIZE = 1000;
const INSET = 40;
const CARDS_LENGTH = 400;

// const SERVER_URL = "https://localhost:7380/game";
// const SERVER_URL =
//   "http://localhost:7380/game/subscribe/891047ef-6b8c-40ad-a5cc-a07bd5705828";
// let eventStream: EventSource | null = null;

// function subscribe() {
//   if (eventStream != null) {
//     eventStream.close();
//   }
//   eventStream = new EventSource(SERVER_URL);
//   eventStream.onmessage = event => {
//     console.log(event.data);
//   };
// }

interface Event {
  begin(): void;
  isFinished(): boolean;
}

interface Point {
  x: number;
  y: number;
}

interface SpriteCard {
  card: Card;
  sprite: PIXI.Sprite;
  hidden: boolean;
}

class SendPassEvent implements Event {
  private tweens: TWEEN.Tween[] = [];
  constructor(private th: TurboHearts, private event: SendPassData) {}

  public begin() {
    const dest = this.getDestination();
    const cards = this.getCards();
    let delay = 0;
    let i = 0;
    const duration = 300;
    const interval = 80;
    for (const card of cards) {
      this.tweens.push(
        new TWEEN.Tween(card.sprite.position)
          .to(
            {
              x: dest.x + dest.offsetX * (i - 1),
              y: dest.y + dest.offsetY * (i - 1)
            },
            1000
          )
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      this.tweens.push(
        new TWEEN.Tween(card.sprite)
          .to({ rotation: dest.rotation }, duration)
          .delay(delay)
          .easing(TWEEN.Easing.Quadratic.Out)
          .start()
      );
      delay += interval;
      i++;
    }
  }

  private getCards() {
    let hand: SpriteCard[];
    switch (this.event.from) {
      case "north":
        hand = this.th.topCards;
        break;
      case "east":
        hand = this.th.rightCards;
        break;
      case "south":
        hand = this.th.bottomCards;
        break;
      case "west":
        hand = this.th.leftCards;
        break;
    }
    if (this.event.cards.length === 0) {
      // pass hidden cards
      return [];
    } else {
      const set = new Set(this.event.cards);
      const cardsToMove = hand.filter(c => set.has(c.card));
      return cardsToMove;
    }
  }

  private getDestination() {
    const r2o2 = Math.sqrt(2) / 2;
    switch (this.event.from) {
      case "north":
        switch (this.th.pass) {
          case "Left":
            return {
              x: SIZE - INSET * 4,
              y: INSET * 4,
              rotation: (Math.PI * 5) / 4,
              offsetX: r2o2 * 25,
              offsetY: r2o2 * 25
            };
        }
        break;
      case "east":
        switch (this.th.pass) {
          case "Left":
            return {
              x: SIZE - INSET * 4,
              y: SIZE - INSET * 4,
              rotation: (Math.PI * 3) / 4,
              offsetX: r2o2 * 25,
              offsetY: -r2o2 * 25
            };
        }
        break;
      case "south":
        switch (this.th.pass) {
          case "Left":
            return {
              x: INSET * 4,
              y: SIZE - INSET * 4,
              rotation: Math.PI / 4,
              offsetX: r2o2 * 25,
              offsetY: r2o2 * 25
            };
        }
        break;
      case "west":
        switch (this.th.pass) {
          case "Left":
            return {
              x: INSET * 4,
              y: INSET * 4,
              rotation: (Math.PI * 3) / 4,
              offsetX: r2o2 * 25,
              offsetY: -r2o2 * 25
            };
        }
        break;
    }
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

class DealEvent implements Event {
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

class TurboHearts {
  public app: PIXI.Application;

  public pass: Pass | undefined;
  public topCards: SpriteCard[];
  public rightCards: SpriteCard[];
  public bottomCards: SpriteCard[];
  public leftCards: SpriteCard[];

  private eventQueue: Event[] = [];
  private currentEvent: Event | undefined = undefined;

  constructor(private canvas: HTMLCanvasElement) {
    const dpr = window.devicePixelRatio;
    this.app = new PIXI.Application({
      view: this.canvas,
      width: SIZE,
      height: SIZE,
      backgroundColor: 0x77a178,
      resolution: dpr
    });
    this.canvas.style.width = SIZE + "px";
    this.canvas.style.height = SIZE + "px";
    this.loadCards();
  }

  private loadCards() {
    for (const card of CARDS) {
      this.app.loader.add(card, `assets/cards/${card}.svg`);
    }
    this.app.loader.load(() => {
      this.app.ticker.add(this.gameLoop);
    });
  }

  public pushEvent(event: Event) {
    this.eventQueue.push(event);
  }

  /**
   * @param delta the number of frames to advance at 60fps
   */
  private gameLoop = (delta: number) => {
    TWEEN.update();
    if (this.currentEvent !== undefined) {
      if (!this.currentEvent.isFinished()) {
        return;
      } else {
        console.log("finished");
        this.currentEvent = undefined;
      }
    }
    if (this.eventQueue.length === 0) {
      return;
    }
    this.currentEvent = this.eventQueue.shift();
    this.currentEvent.begin();
  };
}

function toEvent(th: TurboHearts, event: EventData) {
  switch (event.type) {
    case "deal":
      return new DealEvent(th, event);
    case "send_pass":
      return new SendPassEvent(th, event);
    default:
      return undefined;
  }
}

document.addEventListener("DOMContentLoaded", event => {
  const th = new TurboHearts(
    document.getElementById("turbo-hearts") as HTMLCanvasElement
  );
  // const events = [...TEST_EVENTS];
  for (const event of TEST_EVENTS) {
    const realEvent = toEvent(th, event as EventData);
    if (realEvent !== undefined) {
      th.pushEvent(realEvent);
    }
  }
});
