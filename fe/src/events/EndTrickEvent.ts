import { TurboHearts } from "../game/TurboHearts";
import { EndTrickEventData, Event, Seat, SpriteCard } from "../types";
import { getPlayerAccessor } from "./playerAccessors";
import { pushAll } from "../util/array";
import { animatePile } from "./animations/animations";

export class EndTrickEvent implements Event {
  private finished = false;

  constructor(private th: TurboHearts, private event: EndTrickEventData) {}

  public begin() {
    const pileCards: SpriteCard[] = [];
    ["north", "east", "south", "west"].forEach((seat: Seat) => {
      const player = getPlayerAccessor(this.th.bottomSeat, seat)(this.th);
      player.playCards.forEach(card => {
        pileCards.push(card);
        card.hidden = true;
        card.sprite.texture = this.th.app.loader.resources["BACK"].texture;
        card.sprite.zIndex = 0;
      });
      player.playCards = [];
    });
    this.th.app.stage.sortChildren();
    const winner = getPlayerAccessor(this.th.bottomSeat, this.event.winner)(this.th);
    pushAll(winner.pileCards, pileCards);
    animatePile(this.th, this.event.winner).then(() => {
      this.finished = true;
    });
    this.th.playIndex = 0;
  }

  public isFinished() {
    return this.finished;
  }
}
