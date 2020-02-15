import { Pass, SpriteCard, CARDS, Event, Seat } from "../types";
import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { TABLE_SIZE } from "../const";

export interface Player {
  cards: SpriteCard[];
  limboCards: SpriteCard[];
  chargedCards: SpriteCard[];
}

export class TurboHearts {
  public app: PIXI.Application;

  public pass: Pass | undefined;

  public bottomSeat: Seat = "east";

  public topPlayer: Player;
  public rightPlayer: Player;
  public bottomPlayer: Player;
  public leftPlayer: Player;

  private eventQueue: Event[] = [];
  private currentEvent: Event | undefined = undefined;

  constructor(private canvas: HTMLCanvasElement) {
    const dpr = window.devicePixelRatio;
    this.app = new PIXI.Application({
      view: this.canvas,
      width: TABLE_SIZE,
      height: TABLE_SIZE,
      backgroundColor: 0x77a178,
      resolution: dpr
    });
    this.app.stage.sortableChildren = true;
    this.canvas.style.width = TABLE_SIZE + "px";
    this.canvas.style.height = TABLE_SIZE + "px";
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
