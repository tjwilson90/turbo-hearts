import { TurboHearts } from "../game/TurboHearts";
import { Event, SpriteCard, StartPassingEventData } from "../types";
import { getPlayerAccessor } from "./playerAccessors";

export class StartPassingEvent implements Event {
  public type = "start_passing" as const;

  private finished = false;
  private cardsToPass: SpriteCard[] = [];
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();

  constructor(private th: TurboHearts, private event: StartPassingEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    for (const card of player.cards) {
      card.sprite.interactive = true;
      this.cardMap.set(card.sprite, card);
      card.sprite.addListener("pointertap", this.onClick);
    }
  }

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      console.log("pass card", card.card);
      this.cardsToPass.push(card);
    }
    if (this.cardsToPass.length === 3) {
      console.log("pass ready", card.card);
      for (const card of player.cards) {
        card.sprite.interactive = false;
        card.sprite.removeListener("pointertap", this.onClick);
      }
      this.th.passCards(this.cardsToPass.map(c => c.card)).then(() => {
        this.finished = true;
      });
    }
  };

  public isFinished() {
    return this.finished;
  }
}
