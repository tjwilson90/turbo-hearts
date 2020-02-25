import { Pass, SpriteCard, CARDS, Event, Seat, Card } from "../types";
import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { TABLE_SIZE, TABLE_CENTER_X, TABLE_CENTER_Y, TOP, RIGHT, BOTTOM, LEFT } from "../const";
import { SendPassEvent } from "../events/SendPassEvent";
import { ChargeEvent } from "../events/ChargeEvent";
import { PlaySubmitter } from "./PlaySubmitter";
import { Nameplate } from "../ui/Nameplate";

export interface Player {
  type: "bot" | "human";
  name: string;
  toPlay: boolean;
  cards: SpriteCard[];
  limboCards: SpriteCard[];
  chargedCards: SpriteCard[];
  playCards: SpriteCard[];
  pileCards: SpriteCard[];
  nameplate: Nameplate;
}

function emptyPlayer(x: number, y: number, rotation: number): Player {
  return {
    type: "bot",
    name: "empty",
    toPlay: false,
    cards: [],
    limboCards: [],
    chargedCards: [],
    playCards: [],
    pileCards: [],
    nameplate: new Nameplate("", x, y, rotation)
  };
}

function resetPlayer(player: Player) {
  player.toPlay = false;
  player.cards = [];
  player.limboCards = [];
  player.chargedCards = [];
  player.playCards = [];
  player.pileCards = [];
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

  public topPlayer: Player = emptyPlayer(TOP.x, TOP.y + 43, 0);
  public rightPlayer: Player = emptyPlayer(RIGHT.x - 14, RIGHT.y, -Math.PI / 2);
  public bottomPlayer: Player = emptyPlayer(BOTTOM.x, BOTTOM.y - 14, 0);
  public leftPlayer: Player = emptyPlayer(LEFT.x + 14, LEFT.y, Math.PI / 2);

  public trickNumber = 0;
  public playIndex = 0;

  public replay = true;

  private events: Event[] = [];
  private currentEvent: Event | undefined = undefined;

  public asyncEvent: Event | undefined = undefined;

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
    resetPlayer(this.topPlayer);
    resetPlayer(this.rightPlayer);
    resetPlayer(this.bottomPlayer);
    resetPlayer(this.leftPlayer);
    this.trickNumber = 0;
    this.playIndex = 0;
    this.app.stage.removeChildren();
    this.app.stage.addChild(this.topPlayer.nameplate.container);
    this.app.stage.addChild(this.rightPlayer.nameplate.container);
    this.app.stage.addChild(this.bottomPlayer.nameplate.container);
    this.app.stage.addChild(this.leftPlayer.nameplate.container);
    const bg = new PIXI.Sprite(this.app.loader.resources["background"].texture);
    bg.anchor.set(0.5);
    bg.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
    this.app.stage.addChild(bg);
  }

  public syncToPlay() {
    this.topPlayer.nameplate.setToPlay(this.topPlayer.toPlay);
    this.rightPlayer.nameplate.setToPlay(this.rightPlayer.toPlay);
    this.bottomPlayer.nameplate.setToPlay(this.bottomPlayer.toPlay);
    this.leftPlayer.nameplate.setToPlay(this.leftPlayer.toPlay);
  }

  private loadCards() {
    for (const card of CARDS) {
      this.app.loader.add(card, `assets/cards/${card}.svg`);
    }
    this.app.loader.add("background", `assets/fabric@2x.jpg`);
    this.app.loader.load(() => {
      this.app.ticker.add(this.gameLoop);
    });
  }

  public pushEvent(event: Event) {
    this.events.push(event);
  }

  // TODO
  // The next few functions should be replaced with some sort of
  // InputAction that is separate from the backend event stream.
  // The backend event stream should just be used to update state
  // and kickoff animations. At the end of the replay, the game
  // state should be inspected to see if the the current player
  // needs to make an action. The "asyncEvent" is essentially the
  // current input action the user needs to take.
  //
  private hasEventAfterYourPlay() {
    // HACK: > 1 because of end_replay event
    return this.currentEvent?.type === "play_status" && this.events.length > 1;
  }

  private hasFutureSendPass() {
    return this.currentEvent?.type === "start_passing" && hasSendPassFrom(this.events, this.bottomSeat);
  }

  private hasFutureCharge() {
    return this.currentEvent?.type === "charge_status" && hasChargeFrom(this.events, this.bottomSeat);
  }

  private duplicateAsyncEvent() {
    return this.currentEvent?.type === "charge_status" && this.asyncEvent?.type === "charge_status";
  }

  private gameLoop = () => {
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
    while (this.currentEvent === undefined && this.events.length > 0) {
      this.currentEvent = this.events.shift()!;
      if (this.hasEventAfterYourPlay() || this.hasFutureSendPass() || this.hasFutureCharge()) {
        this.currentEvent = undefined;
      } else {
        this.currentEvent.begin();
        this.currentEvent.transition(this.replay);
        if (this.currentEvent.isFinished()) {
          this.currentEvent = undefined;
        } else {
          break;
        }
      }
    }
  };
}
