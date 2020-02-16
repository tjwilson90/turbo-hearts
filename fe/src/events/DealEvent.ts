import * as PIXI from "pixi.js";
import { BOTTOM, CARD_SCALE, LEFT, RIGHT, TABLE_CENTER_X, TABLE_CENTER_Y, TOP } from "../const";
import { sortSpriteCards } from "../game/sortCards";
import { TurboHearts } from "../game/TurboHearts";
import { Card, DealEventData, Event, PointWithRotation, Position, Seat, SpriteCard } from "../types";
import { animateDeal } from "./animations/animations";

const handAccessors: {
  [bottomSeat: string]: {
    [position: string]: (event: DealEventData) => { seat: Seat; cards: Card[] };
  };
} = {};
handAccessors["north"] = {};
handAccessors["north"]["top"] = event => ({ seat: "south", cards: event.south });
handAccessors["north"]["right"] = event => ({ seat: "west", cards: event.west });
handAccessors["north"]["bottom"] = event => ({ seat: "north", cards: event.north });
handAccessors["north"]["left"] = event => ({ seat: "east", cards: event.east });
handAccessors["east"] = {};
handAccessors["east"]["top"] = event => ({ seat: "west", cards: event.west });
handAccessors["east"]["right"] = event => ({ seat: "north", cards: event.north });
handAccessors["east"]["bottom"] = event => ({ seat: "east", cards: event.east });
handAccessors["east"]["left"] = event => ({ seat: "south", cards: event.south });
handAccessors["south"] = {};
handAccessors["south"]["top"] = event => ({ seat: "north", cards: event.north });
handAccessors["south"]["right"] = event => ({ seat: "east", cards: event.east });
handAccessors["south"]["bottom"] = event => ({ seat: "south", cards: event.south });
handAccessors["south"]["left"] = event => ({ seat: "west", cards: event.west });
handAccessors["west"] = {};
handAccessors["west"]["top"] = event => ({ seat: "east", cards: event.east });
handAccessors["west"]["right"] = event => ({ seat: "south", cards: event.south });
handAccessors["west"]["bottom"] = event => ({ seat: "west", cards: event.west });
handAccessors["west"]["left"] = event => ({ seat: "north", cards: event.north });

function getCardsForPosition(bottomSeat: Seat, position: Position, event: DealEventData) {
  return handAccessors[bottomSeat][position](event);
}

export class DealEvent implements Event {
  private finished = false;

  constructor(private th: TurboHearts, private event: DealEventData) {
    this.th.pass = event.pass;
  }

  private createSpriteCards(hand: Card[], seat: Seat, center: PointWithRotation) {
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
    const top = getCardsForPosition(this.th.bottomSeat, "top", this.event);
    const right = getCardsForPosition(this.th.bottomSeat, "right", this.event);
    const bottom = getCardsForPosition(this.th.bottomSeat, "bottom", this.event);
    const left = getCardsForPosition(this.th.bottomSeat, "left", this.event);
    this.th.topPlayer.cards = this.createSpriteCards(top.cards, top.seat, TOP);
    this.th.rightPlayer.cards = this.createSpriteCards(right.cards, right.seat, RIGHT);
    this.th.bottomPlayer.cards = this.createSpriteCards(bottom.cards, bottom.seat, BOTTOM);
    this.th.leftPlayer.cards = this.createSpriteCards(left.cards, left.seat, LEFT);
    animateDeal(this.th).then(() => {
      this.finished = true;
    });
  }

  public isFinished() {
    return this.finished;
  }
}
