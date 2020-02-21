import { TurboHearts } from "../game/TurboHearts";
import { Event, PlayEventData, SpriteCard } from "../types";
import { pushAll, removeAll } from "../util/array";
import { animateHand, animatePlay, moveHand, movePlay } from "./animations/animations";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";
import { Z_PLAYED_CARDS, Z_TRANSIT_CARDS } from "../const";

export class PlayEvent implements Event {
  public type = "play" as const;

  private finished = false;
  private card: SpriteCard;

  constructor(private th: TurboHearts, private event: PlayEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.seat)(this.th);
    let cards = spriteCardsOf([...player.cards, ...player.chargedCards], [this.event.card]);
    if (cards.length !== 1) {
      cards = player.cards.splice(0, 1);
      this.card = cards[0];
      this.card.card = this.event.card;
      this.card.hidden = false;
      this.card.sprite.texture = this.th.app.loader.resources[this.event.card].texture;
    }
    this.card = cards[0];
    this.card.sprite.zIndex = Z_TRANSIT_CARDS;
    this.th.app.stage.sortChildren();

    pushAll(player.playCards, cards);
    removeAll(player.cards, cards);
    removeAll(player.chargedCards, cards);
  }

  public async transition(instant: boolean) {
    if (instant) {
      moveHand(this.th, this.event.seat);
      movePlay(this.th, this.event.seat);
    } else {
      await Promise.all([animateHand(this.th, this.event.seat), animatePlay(this.th, this.event.seat)]);
    }
    this.card.sprite.zIndex = this.th.playIndex++ + Z_PLAYED_CARDS;
    this.th.app.stage.sortChildren();
    this.finished = true;
  }

  public isFinished() {
    return this.finished;
  }
}
