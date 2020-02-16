import { BOTTOM_LEFT, BOTTOM_RIGHT, TOP_LEFT, TOP_RIGHT } from "../const";
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
passDestinations["Left"] = {};
passDestinations["Left"]["north"] = {};
passDestinations["Left"]["north"]["north"] = BOTTOM_LEFT;
passDestinations["Left"]["north"]["east"] = TOP_LEFT;
passDestinations["Left"]["north"]["south"] = TOP_RIGHT;
passDestinations["Left"]["north"]["west"] = BOTTOM_RIGHT;
passDestinations["Left"]["east"] = {};
passDestinations["Left"]["east"]["north"] = BOTTOM_RIGHT;
passDestinations["Left"]["east"]["east"] = BOTTOM_LEFT;
passDestinations["Left"]["east"]["south"] = TOP_LEFT;
passDestinations["Left"]["east"]["west"] = TOP_RIGHT;
passDestinations["Left"]["south"] = {};
passDestinations["Left"]["south"]["north"] = TOP_RIGHT;
passDestinations["Left"]["south"]["east"] = BOTTOM_RIGHT;
passDestinations["Left"]["south"]["south"] = BOTTOM_LEFT;
passDestinations["Left"]["south"]["west"] = TOP_LEFT;
passDestinations["Left"]["west"] = {};
passDestinations["Left"]["west"]["north"] = TOP_LEFT;
passDestinations["Left"]["west"]["east"] = TOP_RIGHT;
passDestinations["Left"]["west"]["south"] = BOTTOM_RIGHT;
passDestinations["Left"]["west"]["west"] = BOTTOM_LEFT;

export class SendPassEvent implements Event {
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
      // TODO pass hidden cards
      return { cardsToMove: [], cardsToKeep: [] };
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
