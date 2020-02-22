import { TurboHearts } from "../game/TurboHearts";
import { EndTrickEventData, Event, Seat, SpriteCard } from "../types";
import { getPlayerAccessor } from "./playerAccessors";
import { pushAll } from "../util/array";
import { animatePile, movePile } from "./animations/animations";
import { Z_PILE_CARDS, TRICK_COLLECTION_PAUSE } from "../const";
import { sleep } from "./helpers";

export class EndTrickEvent implements Event {
  public type = "end_trick" as const;

  private finished = false;
  private pileCards: SpriteCard[] = [];

  constructor(private th: TurboHearts, private event: EndTrickEventData) {}

  public begin() {
    const winner = getPlayerAccessor(this.th.bottomSeat, this.event.winner)(this.th);
    ["north", "east", "south", "west"].forEach((seat: Seat) => {
      const player = getPlayerAccessor(this.th.bottomSeat, seat)(this.th);
      pushAll(this.pileCards, player.playCards);
      player.playCards = [];
    });
    pushAll(winner.pileCards, this.pileCards);
    this.th.playIndex = 0;
    this.th.trickNumber++;
  }

  public async transition(instant: boolean) {
    if (!instant) {
      await sleep(TRICK_COLLECTION_PAUSE);
    }
    let first = true;
    for (const card of this.pileCards) {
      card.hidden = true;
      card.sprite.texture = this.th.app.loader.resources["BACK"].texture;
      card.sprite.zIndex = Z_PILE_CARDS;
      if (!first) {
        card.sprite.filters = [];
      }
      first = false;
    }
    this.th.app.stage.sortChildren();
    if (instant) {
      movePile(this.th, this.event.winner);
    } else {
      await animatePile(this.th, this.event.winner);
    }
    this.finished = true;
  }

  public isFinished() {
    return this.finished;
  }
}
