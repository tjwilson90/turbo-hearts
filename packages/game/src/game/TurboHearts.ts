import { Pass, SpriteCard, CARDS, Event, Seat, Card } from "../types";
import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { TABLE_SIZE } from "../const";
import { SendPassEvent } from "../events/SendPassEvent";
import { ChargeEvent } from "../events/ChargeEvent";
import { PlaySubmitter } from "./PlaySubmitter";

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

function isChargeEvent(event: Event): event is ChargeEvent {
  return event.type === "charge";
}

function isSendPassEvent(event: Event): event is SendPassEvent {
  return event.type === "send_pass";
}

function hasSendPassFrom(events: Event[], seat: Seat) {
  for (const event of events) {
    if (isSendPassEvent(event) && event.from === seat) {
      return true;
    }
  }
  return false;
}

function hasChargeFrom(events: Event[], seat: Seat) {
  for (const event of events) {
    if (isChargeEvent(event) && event.seat === seat) {
      return true;
    }
  }
  return false;
}

export class TurboHearts {
  public app: PIXI.Application;

  public pass: Pass = "left";

  public bottomSeat: Seat = "south";

  public topPlayer: Player = emptyPlayer();
  public rightPlayer: Player = emptyPlayer();
  public bottomPlayer: Player = emptyPlayer();
  public leftPlayer: Player = emptyPlayer();

  public trickNumber = 0;
  public playIndex = 0;

  private events: Event[] = [];
  private currentEvent: Event | undefined = undefined;

  constructor(private canvas: HTMLCanvasElement, public userId: string, public submitter: PlaySubmitter) {
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

  private hasEventAfterYourPlay() {
    return this.currentEvent.type === "your_play" && this.events.length > 0;
  }

  private hasFutureSendPass() {
    return this.currentEvent.type === "start_passing" && hasSendPassFrom(this.events, this.bottomSeat);
  }

  private hasFutureCharge() {
    return this.currentEvent.type === "start_charging" && hasChargeFrom(this.events, this.bottomSeat);
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
    if (this.hasEventAfterYourPlay() || this.hasFutureSendPass() || this.hasFutureCharge()) {
      this.currentEvent = undefined;
    } else {
      console.log(this.currentEvent, console.log(this.events.length), this.hasFutureCharge());
      this.currentEvent.begin();
    }
  };
}