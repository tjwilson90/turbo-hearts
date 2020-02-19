import { TurboHearts } from "../game/TurboHearts";
import { Event, SpriteCard, StartChargingEventData } from "../types";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class StartChargingEvent implements Event {
  public type = "start_charging" as const;

  private finished = false;
  private chargeableCards: SpriteCard[] = [];
  private cardsToCharge: SpriteCard[] = [];
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();

  constructor(private th: TurboHearts, private event: StartChargingEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.chargeableCards = spriteCardsOf(player.cards, ["TC", "JD", "AH", "QS"]);
    for (const card of this.chargeableCards) {
      card.sprite.interactive = true;
      this.cardMap.set(card.sprite, card);
      card.sprite.addListener("pointertap", this.onClick);
    }
    if (this.chargeableCards.length === 0) {
      this.th.submitter.chargeCards([]);
    }
    // TODO fix for non-classic
    // Passing is non-blocking.
    this.finished = true;
  }

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      console.log("charge card", card.card);
      this.cardsToCharge.push(card);
    }
    if (this.cardsToCharge.length === this.chargeableCards.length) {
      for (const card of player.cards) {
        card.sprite.interactive = false;
        card.sprite.removeListener("pointertap", this.onClick);
      }
      this.th.submitter.chargeCards(this.cardsToCharge.map(c => c.card));
    }
  };

  public isFinished() {
    return this.finished;
  }
}
