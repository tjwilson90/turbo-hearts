import { TurboHearts } from "../game/TurboHearts";
import { Event, YourPlayEventData, SpriteCard } from "../types";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

export class YourPlayEvent implements Event {
  public type = "your_play" as const;

  private finished = false;
  private legalCards: SpriteCard[] = [];
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();

  constructor(private th: TurboHearts, private event: YourPlayEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.legalCards = spriteCardsOf([...player.cards, ...player.chargedCards], this.event.legalPlays);

    for (const card of this.legalCards) {
      this.cardMap.set(card.sprite, card);
      card.sprite.addListener("pointertap", this.onClick);
    }
  }

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      for (const card of this.legalCards) {
        card.sprite.removeListener("pointertap", this.onClick);
      }
      this.th.playCard(card.card).then(() => {
        this.finished = true;
      });
    }
  };

  public isFinished() {
    return true;
  }
}
