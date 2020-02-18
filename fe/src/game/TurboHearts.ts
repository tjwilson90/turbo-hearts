import { Pass, SpriteCard, CARDS, Event, Seat, Card } from "../types";
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
  pileCards: SpriteCard[];
}

function emptyPlayer(): Player {
  return {
    type: "bot",
    name: "empty",
    cards: [],
    limboCards: [],
    chargedCards: [],
    playCards: [],
    pileCards: []
  };
}

export class TurboHearts {
  public app: PIXI.Application;

  public pass: Pass = "left";

  public bottomSeat: Seat = "north";

  public topPlayer: Player = emptyPlayer();
  public rightPlayer: Player = emptyPlayer();
  public bottomPlayer: Player = emptyPlayer();
  public leftPlayer: Player = emptyPlayer();

  public trickNumber = 0;
  public playIndex = 0;

  private events: Event[] = [];
  private currentEvent: Event | undefined = undefined;

  constructor(
    private canvas: HTMLCanvasElement,
    public passCards: (card: Card[]) => Promise<unknown>,
    public playCard: (card: Card) => Promise<unknown>
  ) {
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

  public resetForDeal() {
    this.pass = "left";
    this.topPlayer = emptyPlayer();
    this.rightPlayer = emptyPlayer();
    this.bottomPlayer = emptyPlayer();
    this.leftPlayer = emptyPlayer();
    this.trickNumber = 0;
    this.playIndex = 0;
    this.app.stage.removeChildren();
  }

  private loadCards() {
    for (const card of CARDS) {
      this.app.loader.add(card, `assets/cards/${card}.svg`);
    }
    this.app.loader.load(() => {
      this.app.ticker.add(this.gameLoop);
    });
  }

  public activateCards(legalPlays: Card[]) {}

  public pushEvent(event: Event) {
    this.events.push(event);
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
    if (this.events.length === 0) {
      return;
    }
    this.currentEvent = this.events.shift();
    const isInput = this.currentEvent.type === "your_play" || this.currentEvent.type === "start_passing";
    if (!isInput || this.events.length === 0) {
      console.log(this.currentEvent);
      this.currentEvent.begin();
    } else {
      this.currentEvent = undefined;
    }
  };
}
