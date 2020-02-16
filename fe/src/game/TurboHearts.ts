import { Pass, SpriteCard, CARDS, Event, Seat } from "../types";
import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { TABLE_SIZE } from "../const";

export interface Player {
  type: "bot" | "human";
  name: string;
  cards: SpriteCard[];
  limboCards: SpriteCard[];
  chargedCards: SpriteCard[];
  playCards: SpriteCard[];
}

function emptyPlayer(): Player {
  return {
    type: "bot",
    name: "empty",
    cards: [],
    limboCards: [],
    chargedCards: [],
    playCards: []
  };
}

export class TurboHearts {
  public app: PIXI.Application;

  public pass: Pass | undefined;

  public bottomSeat: Seat = "north";

  public topPlayer: Player = emptyPlayer();
  public rightPlayer: Player = emptyPlayer();
  public bottomPlayer: Player = emptyPlayer();
  public leftPlayer: Player = emptyPlayer();

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
        this.currentEvent = undefined;
      }
    }
    if (this.eventQueue.length === 0) {
      return;
    }
    this.currentEvent = this.eventQueue.shift();
    console.log(this.currentEvent);
    this.currentEvent.begin();
  };
}
