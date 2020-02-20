import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { BOTTOM, CARD_DISPLAY_HEIGHT, FASTER_ANIMATION_DURATION } from "../const";
import { Player, TurboHearts } from "../game/TurboHearts";
import { Card, Event, SpriteCard, YourPlayEventData } from "../types";
import { spriteCardsOf } from "./helpers";
import { getPlayerAccessor } from "./playerAccessors";

function isCharged(player: Player, card: Card) {
  return player.chargedCards.some(c => c.card === card);
}

export class YourPlayEvent implements Event {
  public type = "your_play" as const;

  private finished = false;
  private playableCards: SpriteCard[];
  private legalCards: Set<SpriteCard>;
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();
  private cardTweens: Map<PIXI.Sprite, TWEEN.Tween> = new Map();
  private player: Player;

  constructor(private th: TurboHearts, private event: YourPlayEventData) {}

  public begin() {
    this.player = getPlayerAccessor(this.th.bottomSeat, this.th.bottomSeat)(this.th);
    this.playableCards = [...this.player.cards, ...this.player.chargedCards];
    this.legalCards = new Set(spriteCardsOf(this.playableCards, this.event.legalPlays));
    for (const card of this.playableCards) {
      if (this.legalCards.has(card)) {
        this.cardMap.set(card.sprite, card);
        card.sprite.interactive = true;
        card.sprite.addListener("pointertap", this.onClick);
        card.sprite.addListener("pointerover", this.onOver);
        card.sprite.addListener("pointerout", this.onOut);
        card.sprite.buttonMode = true;
      }
    }
  }

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      for (const card of this.playableCards) {
        if (this.legalCards.has(card)) {
          card.sprite.interactive = false;
          card.sprite.removeListener("pointertap", this.onClick);
          card.sprite.removeListener("pointerover", this.onOver);
          card.sprite.removeListener("pointerout", this.onOut);
          card.sprite.buttonMode = false;
        }
      }
      this.th.submitter.playCard(card.card).then(() => {
        this.finished = true;
      });
    }
  };

  private tweenTo(sprite: PIXI.Sprite, y: number) {
    const existingTween = this.cardTweens.get(sprite);
    if (existingTween !== undefined) {
      existingTween.stop();
    }
    const tween = new TWEEN.Tween(sprite.position).to({ y }, FASTER_ANIMATION_DURATION);
    this.cardTweens.set(sprite, tween);
    tween.start();
  }

  private onOver = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    const bottom = isCharged(this.player, card.card) ? BOTTOM.chargeY : BOTTOM.y;
    this.tweenTo(sprite, bottom - CARD_DISPLAY_HEIGHT / 4);
  };

  private onOut = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    const bottom = isCharged(this.player, card.card) ? BOTTOM.chargeY : BOTTOM.y;
    this.tweenTo(sprite, bottom);
  };

  public isFinished() {
    return this.finished;
  }
}
