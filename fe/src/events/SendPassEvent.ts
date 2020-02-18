import {
  LIMBO_BOTTOM_LEFT,
  LIMBO_TOP_LEFT,
  LIMBO_TOP_RIGHT,
  LIMBO_BOTTOM_RIGHT,
  LIMBO_TOP,
  LIMBO_RIGHT,
  LIMBO_BOTTOM,
  LIMBO_LEFT
} from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Event, PointWithRotation, SendPassEventData } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateCards, animateHand } from "./animations/animations";
import { spriteCardsOf, spriteCardsOfNot } from "./helpers";
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

export class SendPassEvent implements Event {
  public type = "send_pass" as const;

  private finished = false;

  constructor(private th: TurboHearts, private event: SendPassEventData) {}

  public begin() {
    const passDestination = this.getPassDestination();
    const cards = this.updateCards();
    Promise.all([
      animateCards(this.th, cards.cardsToMove, passDestination.x, passDestination.y, passDestination.rotation),
      animateHand(this.th, this.event.from)
    ]).then(() => {
      this.finished = true;
    });
  }

  private updateCards() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.from)(this.th);
    if (this.event.cards.length === 0) {
      const cardsToMove = player.cards.splice(0, 3);
      pushAll(player.limboCards, cardsToMove);
      return { cardsToMove, cardsToKeep: player.cards };
    } else {
      const cardsToMove = spriteCardsOf(player.cards, this.event.cards);
      const cardsToKeep = spriteCardsOfNot(player.cards, this.event.cards);
      removeAll(player.cards, cardsToMove);
      pushAll(player.limboCards, cardsToMove);
      return { cardsToMove, cardsToKeep };
    }
  }

  private getPassDestination() {
    return passDestinations[this.th.pass][this.th.bottomSeat][this.event.from];
  }

  public isFinished() {
    return this.finished;
  }
}
