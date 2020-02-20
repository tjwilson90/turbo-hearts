import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { BOTTOM, CARD_DISPLAY_HEIGHT, FASTER_ANIMATION_DURATION } from "../const";
import { TurboHearts } from "../game/TurboHearts";
import { Event, SpriteCard, StartPassingEventData, Pass } from "../types";
import { Button } from "../ui/Button";
import { getPlayerAccessor } from "./playerAccessors";

const directionText: {[P in Pass]: string} = {
  left: "Left",
  right: "Right",
  across: "Across",
  keeper: "In",
};

export class StartPassingEvent implements Event {
  public type = "start_passing" as const;

  private button: Button;
  private finished = false;
  private cardsToPass: Set<SpriteCard> = new Set();
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();
  private cardTweens: Map<PIXI.Sprite, TWEEN.Tween> = new Map();

  constructor(private th: TurboHearts, private event: StartPassingEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    for (const card of player.cards) {
      card.sprite.interactive = true;
      this.cardMap.set(card.sprite, card);
      card.sprite.addListener("pointertap", this.onClick);
      card.sprite.buttonMode = true;
    }
    this.button = new Button("Pass 3 Cards " + directionText[this.th.pass], this.submitPass);
    this.th.app.stage.addChild(this.button.container);
    this.button.setEnabled(this.cardsToPass.size === 3);
    // Passing is non-blocking.
    this.finished = true;
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
      if (this.cardsToPass.has(card)) {
        this.cardsToPass.delete(card);
        this.tweenTo(card.sprite, BOTTOM.y);
      } else {
        this.cardsToPass.add(card);
        this.tweenTo(card.sprite, BOTTOM.y - CARD_DISPLAY_HEIGHT / 4);
      }
    }
    this.button.setEnabled(this.cardsToPass.size === 3);
  };

  private submitPass = () => {
    const player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.button.setEnabled(false);
    this.th.app.stage.removeChild(this.button.container);
    for (const card of player.cards) {
      card.sprite.interactive = false;
      card.sprite.removeListener("pointertap", this.onClick);
      card.sprite.buttonMode = false;
    }
    this.th.submitter.passCards([...this.cardsToPass.values()].map(c => c.card));
  };

  public isFinished() {
    return this.finished;
  }
}
