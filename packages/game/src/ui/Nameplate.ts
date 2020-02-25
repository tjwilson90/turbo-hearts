import * as PIXI from "pixi.js";
import { Z_NAMEPLATE, TABLE_CENTER_X, TABLE_CENTER_Y } from "../const";

export class Nameplate {
  public container: PIXI.Container = new PIXI.Container();

  constructor(name: string, x: number, y: number, rotation: number) {
    const graphics = new PIXI.Graphics();
    graphics.lineStyle(2, 0xe0e0e0, 1);
    graphics.beginFill(0xf0f0f0);
    graphics.drawRect(0, 0, 150, 24);
    graphics.endFill();
    this.container.addChild(graphics);
    this.container.x = x;
    this.container.y = y;
    this.container.pivot.x = this.container.width / 2;
    this.container.pivot.y = this.container.height;
    this.container.rotation = rotation;
    this.container.zIndex = Z_NAMEPLATE;

    let textEl = new PIXI.Text(name, {
      fontFamily: "Arial",
      fontSize: 14,
      fill: 0x101010,
      lineHeight: 60
    });
    textEl.anchor.set(0.5, 0.5);
    textEl.position.set(this.container.width / 2, this.container.height + 6);
    this.container.addChild(textEl);
  }
}
