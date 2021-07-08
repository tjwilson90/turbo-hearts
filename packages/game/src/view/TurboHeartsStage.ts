import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import {
  BOTTOM,
  CARD_DROP_SHADOW,
  CARD_OVERLAP,
  CARD_SCALE,
  CHARGE_OVERLAP,
  LEFT,
  LIMBO_BOTTOM,
  LIMBO_BOTTOM_LEFT,
  LIMBO_BOTTOM_RIGHT,
  LIMBO_CENTER,
  LIMBO_LEFT,
  LIMBO_RIGHT,
  LIMBO_TOP,
  LIMBO_TOP_LEFT,
  LIMBO_TOP_RIGHT,
  RIGHT,
  TABLE_CENTER_X,
  TABLE_CENTER_Y,
  TABLE_SIZE,
  TOP,
  Z_BACKGROUND,
  Z_CHARGED_CARDS,
  Z_HAND_CARDS,
  Z_LIMBO_CARDS,
  Z_PILE_CARDS,
  Z_PLAYED_CARDS,
  CARD_DISPLAY_HEIGHT,
  FASTER_ANIMATION_DURATION
} from "../const";
import { groupCards } from "../util/groupCards";
import { TurboHearts, Action } from "../game/stateSnapshot";
import { TurboHeartsService } from "../game/TurboHeartsService";
import {
  Animation,
  Card,
  CARDS,
  Pass,
  PlayerCardPositions,
  PlayerSpriteCards,
  PointWithRotation,
  Position,
  Seat,
  SpriteCard
} from "../types";
import { StepAnimation } from "./StepAnimation";
import { spriteCardsOf } from "../util/helpers";
import EventEmitter from "eventemitter3";
import { emptyArray } from "../util/array";
import { POSITION_FOR_BOTTOM_SEAT, addToSeat, subtractSeats } from "../util/seatPositions";

const CHARGEABLE_CARDS: Card[] = ["TC", "JD", "AH", "QS"];

export function createSpriteCard(resources: PIXI.IResourceDictionary, card: Card, hidden: boolean): SpriteCard {
  const sprite = new PIXI.Sprite(hidden ? resources["BACK"].texture : resources[card].texture);
  sprite.scale.set(CARD_SCALE);
  sprite.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
  sprite.anchor.set(0.5, 0.5);
  sprite.filters = [CARD_DROP_SHADOW];
  return {
    card,
    sprite,
    hidden
  };
}

const LAYOUTS_FOR_BOTTOM_SEAT: { [bottomSeat in Seat]: PlayerCardPositions[] } = {
  north: [BOTTOM, LEFT, TOP, RIGHT],
  east: [RIGHT, BOTTOM, LEFT, TOP],
  south: [TOP, RIGHT, BOTTOM, LEFT],
  west: [LEFT, TOP, RIGHT, BOTTOM]
};

const LIMBO_1 = { left: LIMBO_TOP_RIGHT, right: LIMBO_TOP_LEFT, across: LIMBO_BOTTOM, keeper: LIMBO_CENTER };
const LIMBO_2 = { left: LIMBO_BOTTOM_RIGHT, right: LIMBO_TOP_RIGHT, across: LIMBO_LEFT, keeper: LIMBO_CENTER };
const LIMBO_3 = { left: LIMBO_BOTTOM_LEFT, right: LIMBO_BOTTOM_RIGHT, across: LIMBO_TOP, keeper: LIMBO_CENTER };
const LIMBO_4 = { left: LIMBO_TOP_LEFT, right: LIMBO_BOTTOM_LEFT, across: LIMBO_RIGHT, keeper: LIMBO_CENTER };
export const LIMBO_POSITIONS_FOR_BOTTOM_SEAT: {
  [bottomSeat in Seat]: {
    [trueSeat in Seat]: {
      [pass in Pass]: PointWithRotation;
    };
  };
} = {
  north: {
    north: LIMBO_3,
    east: LIMBO_4,
    south: LIMBO_1,
    west: LIMBO_2
  },
  east: {
    north: LIMBO_2,
    east: LIMBO_3,
    south: LIMBO_4,
    west: LIMBO_1
  },
  south: {
    north: LIMBO_1,
    east: LIMBO_2,
    south: LIMBO_3,
    west: LIMBO_4
  },
  west: {
    north: LIMBO_4,
    east: LIMBO_1,
    south: LIMBO_2,
    west: LIMBO_3
  }
};

function emptyPlayerSpriteCards() {
  return {
    hand: [],
    limbo: [],
    charged: [],
    plays: [],
    pile: []
  };
}

export type Mode = "live" | "review";

export function getBottomSeat(state: TurboHearts.StateSnapshot, userId: string) {
  if (state.north.userId === userId) {
    return "north";
  } else if (state.east.userId === userId) {
    return "east";
  } else if (state.south.userId === userId) {
    return "south";
  } else if (state.west.userId === userId) {
    return "west";
  } else {
    return "south";
  }
}

export class TurboHeartsStage {
  public app: PIXI.Application;

  private spectatorMode = false;
  private replay = true;

  private background: PIXI.Sprite | undefined;
  private snapshot: TurboHearts.StateSnapshot | undefined;

  private picked: Set<SpriteCard> = new Set();
  private hovered: SpriteCard | undefined = undefined;
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();
  private cardTweens: Map<PIXI.Sprite, TWEEN.Tween> = new Map();

  private animations: Animation[] = [];
  private runningAnimation: Animation | undefined;

  private cardContainer: PIXI.Container = new PIXI.Container();

  private top: PlayerSpriteCards = emptyPlayerSpriteCards();
  private right: PlayerSpriteCards = emptyPlayerSpriteCards();
  private bottom: PlayerSpriteCards = emptyPlayerSpriteCards();
  private left: PlayerSpriteCards = emptyPlayerSpriteCards();

  private emitter = new EventEmitter();

  private action: Action = "none";
  private legalPlays: Card[] = emptyArray();
  private actionToSet: { action: Action; legalPlays: Card[] } | undefined;

  constructor(
    private canvas: HTMLCanvasElement,
    public userId: string,
    public service: TurboHeartsService,
    private onReady: () => void
  ) {
    const dpr = window.devicePixelRatio;
    this.app = new PIXI.Application({
      view: this.canvas,
      width: TABLE_SIZE,
      height: TABLE_SIZE,
      backgroundColor: 0x77a178,
      resolution: dpr,
      autoStart: false
    });
    this.app.stage.sortableChildren = true;
    this.app.loader.resources;
    this.canvas.style.width = TABLE_SIZE + "px";
    this.canvas.style.height = TABLE_SIZE + "px";
    this.loadCards();
  }

  public enableSpectatorMode = () => {
    console.log("SPECTATOR MODE SET");
    this.spectatorMode = true;
  };

  public endReplay = () => {
    this.replay = false;
    if (this.snapshot !== undefined) {
      this.animations.push(this.snapAnimation());
    }
  };

  public acceptSnapshot = (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => {
    if (!this.replay && this.snapshot !== undefined && event.next.index === this.snapshot.index + 1) {
      this.animations.push(this.getAnimation(this.snapshot, event.next));
    }
    this.snapshot = event.next;
    const player = this.snapshot[getBottomSeat(this.snapshot, this.userId)];
    let legalPlays: Card[] = emptyArray();
    if (player.action === "pass") {
      legalPlays = player.hand;
    } else if (player.action === "charge") {
      legalPlays = CHARGEABLE_CARDS;
    } else if (player.action === "play") {
      legalPlays = player.legalPlays;
    }
    this.setAction(player.action, legalPlays);
  };

  private createCard = (card: Card, hidden: boolean) => {
    const spriteCard = createSpriteCard(this.app.loader.resources, card, hidden);
    this.cardContainer.addChild(spriteCard.sprite);
    return spriteCard;
  };

  private snapAnimation = () => {
    let finished = false;
    return {
      start: () => {
        if (this.snapshot === undefined) {
          finished = true;
          return;
        }
        this.snapToState(this.snapshot);
        setTimeout(() => {
          finished = true;
        }, 50);
      },
      isFinished: () => {
        return finished;
      }
    };
  };

  private getAnimation(previous: TurboHearts.StateSnapshot, next: TurboHearts.StateSnapshot): Animation {
    const noopAnimation = () => ({
      start: () => {},
      isFinished: () => true
    });

    if (next.index - previous.index === 1) {
      if (
        next.event.type === "deal" ||
        next.event.type === "play" ||
        next.event.type === "end_trick" ||
        next.event.type === "recv_pass" ||
        next.event.type === "send_pass" ||
        next.event.type === "claim" ||
        next.event.type === "charge" ||
        next.event.type === "game_complete"
      ) {
        return new StepAnimation(
          this.app.loader.resources,
          this.createCard,
          () => this.cardContainer.sortChildren(),
          this.spectatorMode,
          this.getBottomSeat(next),
          previous,
          next,
          this.top,
          this.right,
          this.bottom,
          this.left
        );
      }
    }
    if (next.event.type === "charge_status" || next.event.type === "pass_status" || next.event.type === "play_status") {
      return noopAnimation();
    }
    if (next.event.type === "sit") {
      return this.snapAnimation();
    }
    console.warn("didn't handle event", next.event)
    return noopAnimation();
  }

  private getBottomSeat(state: TurboHearts.StateSnapshot) {
    return getBottomSeat(state, this.userId);
  }

  private snapToState(state: TurboHearts.StateSnapshot) {
    const bottomSeat = this.getBottomSeat(state);
    const lastPlaySeat = state.event.type === "play_status" ? addToSeat(state.event.nextPlayer, -1) : "north";
    this.cardContainer.removeChildren();

    const layoutHand = (seat: Seat, position: Position, layout: PlayerCardPositions) => {
      const handCards = state[seat].hand.map(c => createSpriteCard(this.app.loader.resources, c, false));
      this[position].hand = handCards;
      const handDests = groupCards(handCards, layout.x, layout.y, layout.rotation, CARD_OVERLAP, false);
      for (let i = 0; i < handCards.length; i++) {
        const card = handCards[i];
        card.sprite.position.set(handDests[i].x, handDests[i].y);
        card.sprite.rotation = layout.rotation;
        card.sprite.zIndex = Z_HAND_CARDS;
        this.cardContainer.addChild(card.sprite);
      }

      const chargeCards = state[seat].charged.map(c => createSpriteCard(this.app.loader.resources, c, false));
      this[position].charged = chargeCards;
      const chargeDests = groupCards(
        chargeCards,
        layout.chargeX,
        layout.chargeY,
        layout.rotation,
        CHARGE_OVERLAP,
        false
      );
      for (let i = 0; i < chargeCards.length; i++) {
        const card = chargeCards[i];
        card.sprite.position.set(chargeDests[i].x, chargeDests[i].y);
        card.sprite.rotation = layout.rotation;
        card.sprite.zIndex = Z_CHARGED_CARDS;
        this.cardContainer.addChild(card.sprite);
      }

      const playCards = state[seat].plays.map(c => createSpriteCard(this.app.loader.resources, c, false));
      const playDests = groupCards(playCards, layout.playX, layout.playY, layout.rotation, CARD_OVERLAP, true);
      this[position].plays = playCards;
      for (let i = 0; i < playCards.length; i++) {
        const card = playCards[i];
        card.sprite.position.set(playDests[i].x, playDests[i].y);
        card.sprite.rotation = layout.rotation;
        card.sprite.zIndex = Z_PLAYED_CARDS - subtractSeats(lastPlaySeat, seat) - 4 * (playCards.length - i);
        this.cardContainer.addChild(card.sprite);
      }

      const pileCards = state[seat].pile.map(c => createSpriteCard(this.app.loader.resources, c, true));
      this[position].pile = pileCards;
      for (let i = 0; i < pileCards.length; i++) {
        const card = pileCards[i];
        card.sprite.texture = this.app.loader.resources["BACK"].texture;
        card.sprite.position.set(layout.pileX, layout.pileY);
        card.sprite.rotation = layout.pileRotation;
        card.sprite.zIndex = Z_PILE_CARDS;
        this.cardContainer.addChild(card.sprite);
      }

      // TODO: hidden?
      const limboCards = state[seat].limbo.map(c => createSpriteCard(this.app.loader.resources, c, false));
      this[position].limbo = limboCards;
      const limboPosition = LIMBO_POSITIONS_FOR_BOTTOM_SEAT[bottomSeat][seat][state.pass];
      const limboDests = groupCards(
        limboCards,
        limboPosition.x,
        limboPosition.y,
        limboPosition.rotation,
        CARD_OVERLAP,
        false
      );
      for (let i = 0; i < limboCards.length; i++) {
        const card = limboCards[i];
        card.sprite.position.set(limboDests[i].x, limboDests[i].y);
        card.sprite.rotation = limboPosition.rotation;
        card.sprite.zIndex = Z_LIMBO_CARDS;
        this.cardContainer.addChild(card.sprite);
      }
    };
    const layouts = LAYOUTS_FOR_BOTTOM_SEAT[bottomSeat];
    const positions = POSITION_FOR_BOTTOM_SEAT[bottomSeat];
    layoutHand("north", positions[0], layouts[0]);
    layoutHand("east", positions[1], layouts[1]);
    layoutHand("south", positions[2], layouts[2]);
    layoutHand("west", positions[3], layouts[3]);

    this.cardContainer.sortChildren();
  }

  private loadCards() {
    for (const card of CARDS) {
      this.app.loader.add(card, `/assets/img/cards/${card}.svg`);
    }
    this.app.loader.add("background", `/assets/img/fabric@2x.jpg`);
    this.app.loader.load(this.initTable);
  }

  private initTable = () => {
    this.background = new PIXI.Sprite(this.app.loader.resources["background"].texture);
    this.background.anchor.set(0.5);
    this.background.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
    this.background.zIndex = Z_BACKGROUND;
    this.app.stage.addChild(this.background);
    this.app.stage.addChild(this.cardContainer);
    requestAnimationFrame(this.gameLoop);
    this.onReady();
  };

  public setAction(action: Action, legalPlays: Card[], immediate: boolean = false) {
    this.actionToSet = { action, legalPlays };
    if (immediate || (!this.replay && this.runningAnimation === undefined && this.animations.length === 0)) {
      this.setActionInternal();
    }
  }

  private setActionInternal() {
    if (this.spectatorMode || this.actionToSet === undefined) {
      return;
    }
    if (this.action === this.actionToSet.action && this.legalPlays === this.actionToSet.legalPlays) {
      this.actionToSet = undefined;
      return;
    }
    this.disableCardInteraction();
    this.action = this.actionToSet.action;
    this.legalPlays = this.actionToSet.legalPlays;
    if (this.action !== "none") {
      this.enableCardInteraction(this.actionToSet.legalPlays);
    }
    this.emitter.emit("action", this.action);
    this.actionToSet = undefined;
  }

  private enableCardInteraction(legalPlays: Card[]) {
    if (this.spectatorMode || this.snapshot === undefined) {
      return;
    }
    // Don't allow interaction with already charged cards.
    if (this.action === "charge") {
      const player = this.snapshot[getBottomSeat(this.snapshot, this.userId)];
      legalPlays = legalPlays.filter(card => !player.charged.includes(card));
    }

    const spriteCards = spriteCardsOf([...this.bottom.hand, ...this.bottom.charged], legalPlays);
    for (const card of spriteCards) {
      this.cardMap.set(card.sprite, card);
      card.sprite.interactive = true;
      card.sprite.buttonMode = true;
      card.sprite.addListener("pointertap", this.onClick);
      card.sprite.addListener("pointerover", this.onOver);
      card.sprite.addListener("pointerout", this.onOut);
    }
  }

  private disableCardInteraction() {
    for (const tween of this.cardTweens.values()) {
      tween.stop();
    }
    for (const sprite of this.cardMap.keys()) {
      sprite.interactive = false;
      sprite.buttonMode = false;
      sprite.removeListener("pointertap", this.onClick);
      sprite.removeListener("pointerover", this.onOver);
      sprite.removeListener("pointerout", this.onOut);
    }
    this.picked.clear();
    this.hovered = undefined;
    this.cardMap.clear();
    this.cardTweens.clear();
    this.emitter.emit("pick", []);
  }

  private tweenTo(sprite: PIXI.Sprite, y: number) {
    const existingTween = this.cardTweens.get(sprite);
    if (existingTween !== undefined) {
      existingTween.stop();
    }
    const tween = new TWEEN.Tween(sprite.position)
      .to({ y }, FASTER_ANIMATION_DURATION)
      .onComplete(() => this.cardTweens.delete(sprite));
    this.cardTweens.set(sprite, tween);
    tween.start();
  }

  private animate(card: SpriteCard) {
    let pos = this.bottom.charged.includes(card) ? BOTTOM.chargeY : BOTTOM.y;
    if (this.picked.has(card)) {
      pos -= 1.33 * CARD_DISPLAY_HEIGHT / 4;
    } else if (this.hovered === card) {
      pos -= CARD_DISPLAY_HEIGHT / 4;
    }
    this.tweenTo(card.sprite, pos);
  }

  private onOver = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    if (card === undefined) {
      throw new Error("missing card to animate");
    }
    this.hovered = card;
    this.animate(card);
  };

  private onOut = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    if (card === undefined) {
      throw new Error("missing card to animate");
    }
    if (this.hovered === card) {
      this.hovered = undefined;
    }
    this.animate(card);
  };

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      if (this.picked.has(card)) {
        this.picked.delete(card);
      } else {
        this.picked.add(card);
      }
      this.animate(card);
      this.emitter.emit(
        "pick",
        Array.from(this.picked).map(sc => sc.card)
      );
    }
  };

  public on(type: "action", listener: (action: Action) => void): void;
  public on(type: "pick", listener: (picks: Card[]) => void): void;
  public on(type: string, listener: EventEmitter.ListenerFn) {
    this.emitter.on(type, listener);
  }

  public off(type: "action", listener: (action: Action) => void): void;
  public off(type: "pick", listener: (picks: Card[]) => void): void;
  public off(type: string, listener: EventEmitter.ListenerFn) {
    this.emitter.off(type, listener);
  }

  private gameLoop = () => {
    requestAnimationFrame(this.gameLoop);
    const tweenUpdate = TWEEN.update();
    if (this.runningAnimation !== undefined || tweenUpdate) {
      this.app.render();
    }
    if (this.replay || (this.runningAnimation !== undefined && !this.runningAnimation.isFinished())) {
      return;
    }

    this.runningAnimation = undefined;
    if (this.animations.length > 0) {
      this.runningAnimation = this.animations.shift()!;
      this.runningAnimation.start();
    } else {
      this.setActionInternal();
    }
  };
}
