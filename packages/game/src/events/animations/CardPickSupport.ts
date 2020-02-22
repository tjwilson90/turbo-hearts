import TWEEN from "@tweenjs/tween.js";
import * as PIXI from "pixi.js";
import { SpriteCard, Point } from "../../types";
import { FASTER_ANIMATION_DURATION, CARD_DISPLAY_HEIGHT } from "../../const";

export class CardPickSupport {
  public picked: Set<SpriteCard> = new Set();
  private hovered: SpriteCard | undefined = undefined;
  private initialPosition: Map<SpriteCard, number> = new Map();
  private cardMap: Map<PIXI.Sprite, SpriteCard> = new Map();
  private cardTweens: Map<PIXI.Sprite, TWEEN.Tween> = new Map();

  constructor(private cards: SpriteCard[], private onPick?: () => void) {
    for (const card of this.cards) {
      this.cardMap.set(card.sprite, card);
      this.initialPosition.set(card, card.sprite.position.y);
      card.sprite.interactive = true;
      card.sprite.buttonMode = true;
      card.sprite.addListener("pointertap", this.onClick);
      card.sprite.addListener("pointerover", this.onOver);
      card.sprite.addListener("pointerout", this.onOut);
    }
  }

  public cleanUp() {
    for (const tween of this.cardTweens.values()) {
      tween.stop();
    }
    for (const card of this.cards) {
      card.sprite.interactive = false;
      card.sprite.buttonMode = false;
      card.sprite.removeListener("pointertap", this.onClick);
      card.sprite.removeListener("pointerover", this.onOver);
      card.sprite.removeListener("pointerout", this.onOut);
    }
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

  private animate(card: SpriteCard) {
    const initialPosition = this.initialPosition.get(card);
    if (initialPosition === undefined) {
      throw new Error("missing card to animate");
    }
    let pos;
    const offset = CARD_DISPLAY_HEIGHT / 4;
    if (this.picked.has(card)) {
      pos = initialPosition - 1.33 * offset;
    } else if (this.hovered === card) {
      pos = initialPosition - offset;
    } else {
      pos = initialPosition;
    }
    this.tweenTo(card.sprite, pos);
  }

  private onOver = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    if (card === undefined) {
      throw new Error("missing card to animate");
    }
    this.hovered = card;
    this.animate(card);
  };

  private onOut = (event: PIXI.interaction.InteractionEvent) => {
    const sprite = event.currentTarget as PIXI.Sprite;
    const card = this.cardMap.get(sprite);
    if (card === undefined) {
      throw new Error("missing card to animate");
    }
    if (this.hovered === card) {
      this.hovered = undefined;
    }
    this.animate(card);
  };

  private onClick = (event: PIXI.interaction.InteractionEvent) => {
    const card = this.cardMap.get(event.currentTarget as PIXI.Sprite);
    if (card !== undefined) {
      if (this.picked.has(card)) {
        this.picked.delete(card);
      } else {
        this.picked.add(card);
      }
      this.animate(card);
      if (this.onPick != null) {
        this.onPick();
      }
    }
  };
}
