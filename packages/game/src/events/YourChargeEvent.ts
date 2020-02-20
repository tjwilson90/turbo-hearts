import TWEEN from "@tweenjs/tween.js";
import { TurboHearts } from "../game/TurboHearts";
import { Event, SpriteCard, YourChargeEventData } from "../types";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";
import { Button } from "../ui/Button";
import { FASTER_ANIMATION_DURATION, BOTTOM, CARD_DISPLAY_HEIGHT } from "../const";

export class YourChargeEvent implements Event {
  public type = "your_charge" as const;

  private button: Button;
  private finished = false;
  private chargeableCards: SpriteCard[] = [];
  private cardsToCharge: Set<SpriteCard> = new Set();
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();
  private cardTweens: Map<PIXI.Sprite, TWEEN.Tween> = new Map();

  constructor(private th: TurboHearts, private event: YourChargeEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.chargeableCards = spriteCardsOf(player.cards, ["TC", "JD", "AH", "QS"]);
    for (const card of this.chargeableCards) {
      card.sprite.interactive = true;
      this.cardMap.set(card.sprite, card);
      card.sprite.addListener("pointertap", this.onClick);
      card.sprite.buttonMode = true;
    }
    this.button = new Button("Charge Cards", this.submitCharge);
    this.button.setEnabled(true);
    this.th.app.stage.addChild(this.button.container);
    if (this.chargeableCards.length === 0) {
      // TODO don't auto submit empty charge
      this.submitCharge();
    }
    this.th.asyncEvent = this;
    this.finished = true;
    // TODO fix for non-classic
  }

  private tweenTo(sprite: PIXI.Sprite, y: number) {
    const existingTween = this.cardTweens.get(sprite);
    if (existingTween !== undefined) {
      existingTween.stop();
    }
    const tween = new TWEEN.Tween(sprite.position).to({ y }, FASTER_ANIMATION_DURATION);
    this.cardTweens.set(sprite, tween);
    tween.start();
  }

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      if (this.cardsToCharge.has(card)) {
        this.cardsToCharge.delete(card);
        this.tweenTo(card.sprite, BOTTOM.y);
      } else {
        this.cardsToCharge.add(card);
        this.tweenTo(card.sprite, BOTTOM.y - CARD_DISPLAY_HEIGHT / 4);
      }
    }
  };

  private submitCharge = () => {
    this.button.setEnabled(false);
    this.th.app.stage.removeChild(this.button.container);
    for (const card of this.chargeableCards) {
      card.sprite.interactive = false;
      card.sprite.removeListener("pointertap", this.onClick);
      card.sprite.buttonMode = false;
    }
    this.th.asyncEvent = undefined;
    this.th.submitter.chargeCards([...this.cardsToCharge.values()].map(c => c.card));
  };

  public isFinished() {
    return this.finished;
  }
}
