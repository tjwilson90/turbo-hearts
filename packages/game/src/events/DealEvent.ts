import * as PIXI from "pixi.js";
import { BOTTOM, CARD_SCALE, LEFT, RIGHT, TABLE_CENTER_X, TABLE_CENTER_Y, TOP } from "../const";
import { sortSpriteCards } from "../game/sortCards";
import { TurboHearts } from "../game/TurboHearts";
import { Card, DealEventData, Event, PointWithRotation, Position, Seat, SpriteCard } from "../types";
import { animateDeal, moveDeal } from "./animations/animations";

const handAccessors: {
  [bottomSeat in Seat]: {
    [position in Position]: (event: DealEventData) => { seat: Seat; cards: Card[] };
  };
} = {
  north: {
    top: event => ({ seat: "south", cards: event.south }),
    right: event => ({ seat: "west", cards: event.west }),
    bottom: event => ({ seat: "north", cards: event.north }),
    left: event => ({ seat: "east", cards: event.east })
  },
  east: {
    top: event => ({ seat: "west", cards: event.west }),
    right: event => ({ seat: "north", cards: event.north }),
    bottom: event => ({ seat: "east", cards: event.east }),
    left: event => ({ seat: "south", cards: event.south })
  },
  south: {
    top: event => ({ seat: "north", cards: event.north }),
    right: event => ({ seat: "east", cards: event.east }),
    bottom: event => ({ seat: "south", cards: event.south }),
    left: event => ({ seat: "west", cards: event.west })
  },
  west: {
    top: event => ({ seat: "east", cards: event.east }),
    right: event => ({ seat: "south", cards: event.south }),
    bottom: event => ({ seat: "west", cards: event.west }),
    left: event => ({ seat: "north", cards: event.north })
  }
};

function getCardsForPosition(bottomSeat: Seat, position: Position, event: DealEventData) {
  return handAccessors[bottomSeat][position](event);
}

export class DealEvent implements Event {
  public type = "deal" as const;

  private finished = false;

  constructor(private th: TurboHearts, private event: DealEventData) {}

  private createSpriteCards(hand: Card[], seat: Seat, center: PointWithRotation) {
    const cards: SpriteCard[] = [];
    if (hand.length === 0) {
      for (let i = 0; i < 13; i++) {
        const card: SpriteCard = {
          card: "BACK",
          sprite: new PIXI.Sprite(this.th.app.loader.resources["BACK"].texture),
          hidden: true
        };
        cards.push(card);
      }
    } else {
      for (const card of hand) {
        const spriteCard = {
          card,
          sprite: new PIXI.Sprite(this.th.app.loader.resources["BACK"].texture),
          hidden: true
        };
        spriteCard.sprite.interactive = true;
        cards.push(spriteCard);
      }
    }
    sortSpriteCards(cards);
    for (const card of cards) {
      card.sprite.scale.set(CARD_SCALE);
      card.sprite.position.set(TABLE_CENTER_X, TABLE_CENTER_Y);
      card.sprite.anchor.set(0.5, 0.5);
      card.sprite.rotation = -Math.PI;
      card.hidden = false;
      this.th.app.stage.addChild(card.sprite);
    }

    return cards;
  }

  public begin() {
    this.th.resetForDeal();
    this.th.pass = this.event.pass;
    const top = getCardsForPosition(this.th.bottomSeat, "top", this.event);
    const right = getCardsForPosition(this.th.bottomSeat, "right", this.event);
    const bottom = getCardsForPosition(this.th.bottomSeat, "bottom", this.event);
    const left = getCardsForPosition(this.th.bottomSeat, "left", this.event);
    this.th.topPlayer.cards = this.createSpriteCards(top.cards, top.seat, TOP);
    this.th.rightPlayer.cards = this.createSpriteCards(right.cards, right.seat, RIGHT);
    this.th.bottomPlayer.cards = this.createSpriteCards(bottom.cards, bottom.seat, BOTTOM);
    this.th.leftPlayer.cards = this.createSpriteCards(left.cards, left.seat, LEFT);
  }

  public async transition(instant: boolean) {
    if (instant) {
      moveDeal(this.th);
    } else {
      await animateDeal(this.th);
    }
    this.finished = true;
  }

  public isFinished() {
    return this.finished;
  }
}
