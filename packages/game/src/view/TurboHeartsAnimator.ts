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
  Z_PLAYED_CARDS,
  LIMBO_BOTTOM_LEFT,
  LIMBO_BOTTOM_RIGHT,
  LIMBO_CENTER,
  LIMBO_TOP,
  LIMBO_TOP_LEFT,
  LIMBO_RIGHT,
  LIMBO_LEFT,
  LIMBO_TOP_RIGHT,
  LIMBO_BOTTOM,
  CARD_DROP_SHADOW
} from "../const";
import { groupCards } from "../events/groupCards";
import { PlaySubmitter } from "../game/PlaySubmitter";
import { TurboHearts } from "../game/stateSnapshot";
import {
  Animation,
  Card,
  CARDS,
  PlayerCardPositions,
  Seat,
  Pass,
  PointWithRotation,
  SpriteCard,
  PlayerSpriteCards,
  Position
} from "../types";
import { StepAnimation } from "./StepAnimation";

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

const POSITION_FOR_BOTTOM_SEAT: { [bottomSeat in Seat]: Position[] } = {
  north: ["bottom", "left", "top", "right"],
  east: ["right", "bottom", "left", "top"],
  south: ["top", "right", "bottom", "left"],
  west: ["left", "top", "right", "bottom"]
};

const LIMBO_1 = { left: LIMBO_TOP_RIGHT, right: LIMBO_TOP_LEFT, across: LIMBO_BOTTOM, keeper: LIMBO_CENTER };
const LIMBO_2 = { left: LIMBO_BOTTOM_RIGHT, right: LIMBO_TOP_RIGHT, across: LIMBO_LEFT, keeper: LIMBO_CENTER };
const LIMBO_3 = { left: LIMBO_BOTTOM_LEFT, right: LIMBO_BOTTOM_RIGHT, across: LIMBO_TOP, keeper: LIMBO_CENTER };
const LIMBO_4 = { left: LIMBO_TOP_LEFT, right: LIMBO_BOTTOM_LEFT, across: LIMBO_RIGHT, keeper: LIMBO_CENTER };
const LIMBO_POSITIONS_FOR_BOTTOM_SEAT: {
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

export class TurboHeartsAnimator {
  public app: PIXI.Application;

  private background: PIXI.Sprite | undefined;
  private snapshots: TurboHearts.StateSnapshot[] = [];
  private currentSnapshotIndex = -1;

  private animations: Animation[] = [];
  private runningAnimation: Animation | undefined;

  private cardContainer: PIXI.Container = new PIXI.Container();

  private top: PlayerSpriteCards = emptyPlayerSpriteCards();
  private right: PlayerSpriteCards = emptyPlayerSpriteCards();
  private bottom: PlayerSpriteCards = emptyPlayerSpriteCards();
  private left: PlayerSpriteCards = emptyPlayerSpriteCards();

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

  private createCard = (card: Card, hidden: boolean) => {
    const spriteCard = createSpriteCard(this.app.loader.resources, card, hidden);
    this.cardContainer.addChild(spriteCard.sprite);
    return spriteCard;
  };

  private getAnimation(previous: TurboHearts.StateSnapshot, next: TurboHearts.StateSnapshot): Animation {
    const snapAnimation = () => {
      let finished = false;
      return {
        start: () => {
          this.snapToState(next);
          setTimeout(() => {
            finished = true;
          }, 50);
        },
        isFinished: () => {
          return finished;
        }
      };
    };

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
        next.event.type === "charge"
      ) {
        return new StepAnimation(
          this.app.loader.resources,
          this.createCard,
          () => this.cardContainer.sortChildren(),
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
    return snapAnimation();
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
    this.cardContainer.removeChildren();

    // TODO: nameplates
    // TODO: to play indicator

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
      const playDests = groupCards(playCards, layout.playX, layout.playY, layout.rotation, CARD_OVERLAP, false);
      this[position].plays = playCards;
      for (let i = 0; i < playCards.length; i++) {
        const card = playCards[i];
        card.sprite.position.set(playDests[i].x, playDests[i].y);
        card.sprite.rotation = layout.rotation;
        card.sprite.zIndex = Z_PLAYED_CARDS;
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
        card.sprite.zIndex = Z_HAND_CARDS;
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
    this.app.stage.addChild(this.cardContainer);
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
