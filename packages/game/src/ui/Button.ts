import * as PIXI from "pixi.js";

import { TABLE_CENTER_X, TABLE_SIZE, CARD_DISPLAY_HEIGHT, CARD_MARGIN, Z_BUTTON } from "../const";

export class Button {
  public container: PIXI.Container = new PIXI.Container();

  constructor(text: string, y: number, private callback: () => void) {
    const graphics = new PIXI.Graphics();
    graphics.lineStyle(2, 0xe0e0e0, 1);
    graphics.beginFill(0xf0f0f0);
    graphics.drawRect(0, 0, 240, 60);
    graphics.endFill();
    this.container.addChild(graphics);
    this.container.x = TABLE_CENTER_X;
    this.container.y = y;
    this.container.pivot.x = this.container.width / 2;
    this.container.pivot.y = this.container.height;
    this.container.zIndex = Z_BUTTON;

    let textEl = new PIXI.Text(text, {
      fontFamily: "Arial",
      fontSize: 24,
      fill: 0x101010,
      lineHeight: 60
    });
    textEl.anchor.set(0.5, 0.5);
    textEl.position.set(this.container.width / 2, (this.container.height * 3) / 4);
    this.container.addChild(textEl);
  }

  public setEnabled(enabled: boolean) {
    this.container.interactive = true;
    this.container.buttonMode = enabled;
    this.container.alpha = enabled ? 1.0 : 0.5;
    if (enabled) {
      this.container.addListener("pointertap", this.callback);
    } else {
      this.container.removeListener("pointertap", this.callback);
    }
  }
}
