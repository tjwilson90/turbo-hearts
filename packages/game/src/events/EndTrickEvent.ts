import { TurboHearts } from "../game/TurboHearts";
import { EndTrickEventData, Event, Seat, SpriteCard } from "../types";
import { getPlayerAccessor } from "./playerAccessors";
import { pushAll } from "../util/array";
import { animatePile } from "./animations/animations";
import { Z_PILE_CARDS, TRICK_COLLECTION_PAUSE } from "../const";
import { sleep } from "./helpers";

export class EndTrickEvent implements Event {
  public type = "end_trick" as const;

  private finished = false;

  constructor(private th: TurboHearts, private event: EndTrickEventData) {}

  public async begin() {
    await sleep(TRICK_COLLECTION_PAUSE);
    const pileCards: SpriteCard[] = [];
    ["north", "east", "south", "west"].forEach((seat: Seat) => {
      const player = getPlayerAccessor(this.th.bottomSeat, seat)(this.th);
      player.playCards.forEach(card => {
        pileCards.push(card);
        card.hidden = true;
        card.sprite.texture = this.th.app.loader.resources["BACK"].texture;
        card.sprite.zIndex = Z_PILE_CARDS;
      });
      player.playCards = [];
    });
    this.th.app.stage.sortChildren();
    const winner = getPlayerAccessor(this.th.bottomSeat, this.event.winner)(this.th);
    pushAll(winner.pileCards, pileCards);
    this.th.playIndex = 0;
    this.th.trickNumber++;
    await animatePile(this.th, this.event.winner);
    this.finished = true;
  }

  public isFinished() {
    return this.finished;
  }
}
