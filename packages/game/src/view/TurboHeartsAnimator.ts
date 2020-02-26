import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import {
  BOTTOM,
  CARD_OVERLAP,
  CARD_SCALE,
  CHARGE_OVERLAP,
  LEFT,
  RIGHT,
  TABLE_CENTER_X,
  TABLE_CENTER_Y,
  TABLE_SIZE,
  TOP,
  Z_BACKGROUND,
  Z_CHARGED_CARDS,
  Z_HAND_CARDS,
  Z_PILE_CARDS,
  Z_PLAYED_CARDS
} from "../const";
import { groupCards } from "../events/groupCards";
import { PlaySubmitter } from "../game/PlaySubmitter";
import { TurboHearts } from "../game/stateSnapshot";
import { Card, CARDS, PlayerCardPositions, Seat } from "../types";

function createSpriteCard(resources: PIXI.IResourceDictionary, card: Card, hidden: boolean) {
  const sprite = new PIXI.Sprite(hidden ? resources["BACK"].texture : resources[card].texture);
  sprite.scale.set(CARD_SCALE);
  sprite.anchor.set(0.5, 0.5);
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

export interface Animation {
  start(): void;
  isFinished(): boolean;
}

export class TurboHeartsAnimator {
  public app: PIXI.Application;

  private background: PIXI.Sprite | undefined;
  private snapshots: TurboHearts.StateSnapshot[] = [];
  private currentSnapshotIndex = -1;

  private animations: Animation[] = [];
  private runningAnimation: Animation | undefined;

  constructor(
    private canvas: HTMLCanvasElement,
    public userId: string,
    public submitter: PlaySubmitter,
    private onReady: () => void
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
    this.app.loader.resources;
    this.canvas.style.width = TABLE_SIZE + "px";
    this.canvas.style.height = TABLE_SIZE + "px";
    this.loadCards();
  }

  public acceptSnapshot = (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => {
    this.snapshots.push(event.next);
  };

  private getAnimation(previous: TurboHearts.StateSnapshot, next: TurboHearts.StateSnapshot): Animation {
    let finished = false;
    return {
      start: () => {
        setTimeout(() => {
          this.snapToState(next);
          finished = true;
        }, 500);
      },
      isFinished: () => {
        return finished;
      }
    };
  }

  private getBottomSeat(state: TurboHearts.StateSnapshot) {
    if (state.north.name === this.userId) {
      return "north";
    } else if (state.east.name === this.userId) {
      return "east";
    } else if (state.south.name === this.userId) {
      return "south";
    } else if (state.west.name === this.userId) {
      return "west";
    } else {
      return "south";
    }
  }

  private snapToState(state: TurboHearts.StateSnapshot) {
    const bottomSeat = this.getBottomSeat(state);
    this.app.stage.removeChildren();
    this.app.stage.addChild(this.background!);

    // TODO: nameplates
    // TODO: to play indicator

    const layoutHand = (seat: Seat, position: PlayerCardPositions) => {
      const handCards = state[seat].hand.map(c => createSpriteCard(this.app.loader.resources, c, false));
      const handDests = groupCards(handCards, position.x, position.y, position.rotation, CARD_OVERLAP, false);
      for (let i = 0; i < handCards.length; i++) {
        const card = handCards[i];
        card.sprite.position.set(handDests[i].x, handDests[i].y);
        card.sprite.rotation = position.rotation;
        card.sprite.zIndex = Z_HAND_CARDS;
        this.app.stage.addChild(card.sprite);
      }

      const chargeCards = state[seat].charged.map(c => createSpriteCard(this.app.loader.resources, c, false));
      const chargeDests = groupCards(
        chargeCards,
        position.chargeX,
        position.chargeY,
        position.rotation,
        CHARGE_OVERLAP,
        false
      );
      for (let i = 0; i < chargeCards.length; i++) {
        const card = chargeCards[i];
        card.sprite.position.set(chargeDests[i].x, chargeDests[i].y);
        card.sprite.rotation = position.rotation;
        card.sprite.zIndex = Z_CHARGED_CARDS;
        this.app.stage.addChild(card.sprite);
      }

      const playCards = state[seat].plays.map(c => createSpriteCard(this.app.loader.resources, c, false));
      const playDests = groupCards(playCards, position.playX, position.playY, position.rotation, CARD_OVERLAP, false);
      for (let i = 0; i < playCards.length; i++) {
        const card = playCards[i];
        card.sprite.position.set(playDests[i].x, playDests[i].y);
        card.sprite.rotation = position.rotation;
        card.sprite.zIndex = Z_PLAYED_CARDS;
        this.app.stage.addChild(card.sprite);
      }

      const pileCards = state[seat].pile.map(c => createSpriteCard(this.app.loader.resources, c, true));
      for (let i = 0; i < pileCards.length; i++) {
        const card = pileCards[i];
        card.sprite.position.set(position.pileX, position.pileY);
        card.sprite.rotation = position.pileRotation;
        card.sprite.zIndex = Z_PILE_CARDS;
        this.app.stage.addChild(card.sprite);
      }
    };
    const layouts = LAYOUTS_FOR_BOTTOM_SEAT[bottomSeat];
    layoutHand("north", layouts[0]);
    layoutHand("east", layouts[1]);
    layoutHand("south", layouts[2]);
    layoutHand("west", layouts[3]);

    this.app.stage.sortChildren();
  }

  private loadCards() {
    for (const card of CARDS) {
      this.app.loader.add(card, `assets/cards/${card}.svg`);
    }
    this.app.loader.add("background", `assets/fabric@2x.jpg`);
    this.app.loader.load(this.initTable);
  }

  private initTable = () => {
    this.background = new PIXI.Sprite(this.app.loader.resources["background"].texture);
    this.background.anchor.set(0.5);
    this.background.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
    this.background.zIndex = Z_BACKGROUND;
    this.app.stage.addChild(this.background);
    this.app.ticker.add(this.gameLoop);
    this.onReady();
  };

  private gameLoop = () => {
    TWEEN.update();
    if (this.runningAnimation !== undefined && !this.runningAnimation.isFinished()) {
      return;
    }

    if (this.animations.length > 0) {
      this.runningAnimation = this.animations.shift()!;
      this.runningAnimation.start();
      return;
    }

    if (this.currentSnapshotIndex === -1) {
      if (this.snapshots.length > 0) {
        this.snapToState(this.snapshots[0]);
        this.currentSnapshotIndex = 0;
      }
      return;
    }

    if (this.currentSnapshotIndex < this.snapshots.length - 1) {
      const current = this.snapshots[this.currentSnapshotIndex];
      const next = this.snapshots[this.currentSnapshotIndex + 1];
      const animation = this.getAnimation(current, next);
      this.animations.push(animation);
      this.currentSnapshotIndex++;
    }
  };
}
