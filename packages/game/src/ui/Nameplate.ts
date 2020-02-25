import * as PIXI from "pixi.js";
import { Z_NAMEPLATE, TABLE_CENTER_X, TABLE_CENTER_Y } from "../const";

export class Nameplate {
  public container: PIXI.Container = new PIXI.Container();

  private nameText: PIXI.Text;
  private toPlayIndicator: PIXI.Graphics;
  private toPlay = false;

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

    this.nameText = new PIXI.Text(name, {
      fontFamily: "Arial",
      fontSize: 14,
      fill: 0x101010,
      lineHeight: 60
    });
    this.nameText.anchor.set(0.5, 0.5);
    this.nameText.position.set(this.container.width / 2, this.container.height + 6);
    this.toPlayIndicator = new PIXI.Graphics();
    this.toPlayIndicator.lineStyle(1, 0xff0000, 1);
    this.toPlayIndicator.beginFill(0xff8080);
    this.toPlayIndicator.drawRect(4, 4, 16, 16);
    this.toPlayIndicator.endFill();
    this.container.addChild(this.nameText);
  }

  public setName(name: string) {
    this.nameText.text = name;
  }

  public setToPlay(state: boolean) {
    if (this.toPlay === state) {
      return;
    }
    if (state) {
      this.container.addChild(this.toPlayIndicator);
    } else {
      this.container.removeChild(this.toPlayIndicator);
    }
    this.toPlay = state;
  }
}
