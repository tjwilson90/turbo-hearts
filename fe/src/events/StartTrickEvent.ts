import * as PIXI from "pixi.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, StartTrickEventData } from "../types";
import { getPlayerAccessor } from "./playerAccessors";

export class StartTrickEvent implements Event {
  public type = "start_trick" as const;

  constructor(private th: TurboHearts, private event: StartTrickEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    if (this.th.trickNumber === 0) {
      [...player.cards, ...player.chargedCards].forEach(card => {
        card.sprite.interactive = true;
        card.sprite.addListener("pointerover", this.onOver);
        card.sprite.addListener("pointerout", this.onOut);
      });
    }
  }

  private onOver = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    sprite.position.y -= 50;
  };

  private onOut = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    sprite.position.y += 50;
  };

  public isFinished() {
    return true;
  }
}
