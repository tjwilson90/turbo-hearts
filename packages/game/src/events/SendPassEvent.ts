import {
  CARD_OVERLAP,
  LIMBO_BOTTOM,
  LIMBO_BOTTOM_LEFT,
  LIMBO_BOTTOM_RIGHT,
  LIMBO_CENTER,
  LIMBO_LEFT,
  LIMBO_RIGHT,
  LIMBO_TOP,
  LIMBO_TOP_LEFT,
  LIMBO_TOP_RIGHT
} from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Event, PointWithRotation, Seat, SendPassEventData, SpriteCard } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateCards, animateHand, moveCards, moveHand } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

const passDestinations: {
  [pass: string]: {
    [bottomSeat: string]: { [passFrom: string]: PointWithRotation };
  };
} = {};
passDestinations["left"] = {};
passDestinations["left"]["north"] = {};
passDestinations["left"]["north"]["north"] = LIMBO_BOTTOM_LEFT;
passDestinations["left"]["north"]["east"] = LIMBO_TOP_LEFT;
passDestinations["left"]["north"]["south"] = LIMBO_TOP_RIGHT;
passDestinations["left"]["north"]["west"] = LIMBO_BOTTOM_RIGHT;
passDestinations["left"]["east"] = {};
passDestinations["left"]["east"]["north"] = LIMBO_BOTTOM_RIGHT;
passDestinations["left"]["east"]["east"] = LIMBO_BOTTOM_LEFT;
passDestinations["left"]["east"]["south"] = LIMBO_TOP_LEFT;
passDestinations["left"]["east"]["west"] = LIMBO_TOP_RIGHT;
passDestinations["left"]["south"] = {};
passDestinations["left"]["south"]["north"] = LIMBO_TOP_RIGHT;
passDestinations["left"]["south"]["east"] = LIMBO_BOTTOM_RIGHT;
passDestinations["left"]["south"]["south"] = LIMBO_BOTTOM_LEFT;
passDestinations["left"]["south"]["west"] = LIMBO_TOP_LEFT;
passDestinations["left"]["west"] = {};
passDestinations["left"]["west"]["north"] = LIMBO_TOP_LEFT;
passDestinations["left"]["west"]["east"] = LIMBO_TOP_RIGHT;
passDestinations["left"]["west"]["south"] = LIMBO_BOTTOM_RIGHT;
passDestinations["left"]["west"]["west"] = LIMBO_BOTTOM_LEFT;

passDestinations["right"] = {};
passDestinations["right"]["north"] = {};
passDestinations["right"]["north"]["north"] = LIMBO_BOTTOM_RIGHT;
passDestinations["right"]["north"]["east"] = LIMBO_BOTTOM_LEFT;
passDestinations["right"]["north"]["south"] = LIMBO_TOP_LEFT;
passDestinations["right"]["north"]["west"] = LIMBO_TOP_RIGHT;
passDestinations["right"]["east"] = {};
passDestinations["right"]["east"]["north"] = LIMBO_TOP_RIGHT;
passDestinations["right"]["east"]["east"] = LIMBO_BOTTOM_RIGHT;
passDestinations["right"]["east"]["south"] = LIMBO_BOTTOM_LEFT;
passDestinations["right"]["east"]["west"] = LIMBO_TOP_LEFT;
passDestinations["right"]["south"] = {};
passDestinations["right"]["south"]["north"] = LIMBO_TOP_LEFT;
passDestinations["right"]["south"]["east"] = LIMBO_TOP_RIGHT;
passDestinations["right"]["south"]["south"] = LIMBO_BOTTOM_RIGHT;
passDestinations["right"]["south"]["west"] = LIMBO_BOTTOM_LEFT;
passDestinations["right"]["west"] = {};
passDestinations["right"]["west"]["north"] = LIMBO_BOTTOM_LEFT;
passDestinations["right"]["west"]["east"] = LIMBO_TOP_LEFT;
passDestinations["right"]["west"]["south"] = LIMBO_TOP_RIGHT;
passDestinations["right"]["west"]["west"] = LIMBO_BOTTOM_RIGHT;

passDestinations["across"] = {};
passDestinations["across"]["north"] = {};
passDestinations["across"]["north"]["north"] = LIMBO_TOP;
passDestinations["across"]["north"]["east"] = LIMBO_RIGHT;
passDestinations["across"]["north"]["south"] = LIMBO_BOTTOM;
passDestinations["across"]["north"]["west"] = LIMBO_LEFT;
passDestinations["across"]["east"] = {};
passDestinations["across"]["east"]["north"] = LIMBO_LEFT;
passDestinations["across"]["east"]["east"] = LIMBO_TOP;
passDestinations["across"]["east"]["south"] = LIMBO_RIGHT;
passDestinations["across"]["east"]["west"] = LIMBO_BOTTOM;
passDestinations["across"]["south"] = {};
passDestinations["across"]["south"]["north"] = LIMBO_BOTTOM;
passDestinations["across"]["south"]["east"] = LIMBO_LEFT;
passDestinations["across"]["south"]["south"] = LIMBO_TOP;
passDestinations["across"]["south"]["west"] = LIMBO_RIGHT;
passDestinations["across"]["west"] = {};
passDestinations["across"]["west"]["north"] = LIMBO_RIGHT;
passDestinations["across"]["west"]["east"] = LIMBO_BOTTOM;
passDestinations["across"]["west"]["south"] = LIMBO_LEFT;
passDestinations["across"]["west"]["west"] = LIMBO_TOP;

passDestinations["keeper"] = {};
passDestinations["keeper"]["north"] = {};
passDestinations["keeper"]["north"]["north"] = LIMBO_CENTER;
passDestinations["keeper"]["north"]["east"] = LIMBO_CENTER;
passDestinations["keeper"]["north"]["south"] = LIMBO_CENTER;
passDestinations["keeper"]["north"]["west"] = LIMBO_CENTER;
passDestinations["keeper"]["east"] = {};
passDestinations["keeper"]["east"]["north"] = LIMBO_CENTER;
passDestinations["keeper"]["east"]["east"] = LIMBO_CENTER;
passDestinations["keeper"]["east"]["south"] = LIMBO_CENTER;
passDestinations["keeper"]["east"]["west"] = LIMBO_CENTER;
passDestinations["keeper"]["south"] = {};
passDestinations["keeper"]["south"]["north"] = LIMBO_CENTER;
passDestinations["keeper"]["south"]["east"] = LIMBO_CENTER;
passDestinations["keeper"]["south"]["south"] = LIMBO_CENTER;
passDestinations["keeper"]["south"]["west"] = LIMBO_CENTER;
passDestinations["keeper"]["west"] = {};
passDestinations["keeper"]["west"]["north"] = LIMBO_CENTER;
passDestinations["keeper"]["west"]["east"] = LIMBO_CENTER;
passDestinations["keeper"]["west"]["south"] = LIMBO_CENTER;
passDestinations["keeper"]["west"]["west"] = LIMBO_CENTER;

export class SendPassEvent implements Event {
  public type = "send_pass" as const;
  public from: Seat;

  private finished = false;
  private cardsToMove: SpriteCard[] = [];

  constructor(private th: TurboHearts, private event: SendPassEventData) {
    this.from = event.from;
  }

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.from)(this.th);
    if (this.event.cards.length === 0) {
      this.cardsToMove = player.cards.splice(0, 3);
      pushAll(player.limboCards, this.cardsToMove);
    } else {
      this.cardsToMove = spriteCardsOf(player.cards, this.event.cards);
      removeAll(player.cards, this.cardsToMove);
      pushAll(player.limboCards, this.cardsToMove);
      for (const card of this.cardsToMove) {
        card.hidden = true;
        card.sprite.texture = this.th.app.loader.resources["BACK"].texture;
      }
    }
  }

  public async transition(instant: boolean) {
    const passDestination = passDestinations[this.th.pass][this.th.bottomSeat][this.event.from];
    const spread = this.th.pass === "keeper" ? 0 : CARD_OVERLAP;
    if (instant) {
      moveCards(this.th, this.cardsToMove, passDestination.x, passDestination.y, passDestination.rotation, spread);
      moveHand(this.th, this.event.from);
    } else {
      await Promise.all([
        animateCards(this.th, this.cardsToMove, passDestination.x, passDestination.y, passDestination.rotation, spread),
        await animateHand(this.th, this.event.from)
      ]);
    }
    this.finished = true;
  }

  public isFinished() {
    return this.finished;
  }
}
