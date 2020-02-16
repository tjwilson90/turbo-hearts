import { TurboHearts } from "../game/TurboHearts";
import { ChargeEventData, Event } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateCharges, animateHand } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class ChargeEvent implements Event {
  private finished = false;

  constructor(private th: TurboHearts, private event: ChargeEventData) {}

  public begin() {
    if (this.event.cards.length === 0) {
      this.finished = true;
      return;
    }
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.seat)(this.th);

    const chargeCards = spriteCardsOf(player.cards, this.event.cards);
    if (chargeCards.length !== this.event.cards.length) {
      // Pull from unknown hand
      const charged = player.cards.splice(0, this.event.cards.length);
      for (let i = 0; i < charged.length; i++) {
        charged[i].card = this.event.cards[i];
        charged[i].hidden = false;
      }
      pushAll(player.chargedCards, charged);
    } else {
      // Pull from known hand
      removeAll(player.cards, chargeCards);
      pushAll(player.chargedCards, chargeCards);
    }

    Promise.all([animateHand(this.th, this.event.seat), animateCharges(this.th, this.event.seat)]).then(() => {
      this.finished = true;
    });
  }

  public isFinished() {
    return this.finished;
  }
}
