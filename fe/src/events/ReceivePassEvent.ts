import { TurboHearts } from "../game/TurboHearts";
import { Event, ReceivePassEventData, SpriteCard } from "../types";
import { animateHand } from "./animations/animations";
import { getPlayerAccessor } from "./playerAccessors";
import { sortSpriteCards } from "../game/sortCards";
import { Z_HAND_CARDS } from "../const";

const limboSources: {
  [pass: string]: {
    [bottomSeat: string]: {
      [passFrom: string]: (th: TurboHearts) => SpriteCard[];
    };
  };
} = {};
limboSources["left"] = {};
limboSources["left"]["north"] = {};
limboSources["left"]["north"]["north"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["left"]["north"]["east"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["left"]["north"]["south"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["left"]["north"]["west"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["left"]["east"] = {};
limboSources["left"]["east"]["north"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["left"]["east"]["east"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["left"]["east"]["south"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["left"]["east"]["west"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["left"]["south"] = {};
limboSources["left"]["south"]["north"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["left"]["south"]["east"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["left"]["south"]["south"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["left"]["south"]["west"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["left"]["west"] = {};
limboSources["left"]["west"]["north"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["left"]["west"]["east"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["left"]["west"]["south"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["left"]["west"]["west"] = (th: TurboHearts) => th.rightPlayer.limboCards;

limboSources["right"] = {};
limboSources["right"]["north"] = {};
limboSources["right"]["north"]["north"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["right"]["north"]["east"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["right"]["north"]["south"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["right"]["north"]["west"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["right"]["east"] = {};
limboSources["right"]["east"]["north"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["right"]["east"]["east"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["right"]["east"]["south"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["right"]["east"]["west"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["right"]["south"] = {};
limboSources["right"]["south"]["north"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["right"]["south"]["east"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["right"]["south"]["south"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["right"]["south"]["west"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["right"]["west"] = {};
limboSources["right"]["west"]["north"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["right"]["west"]["east"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["right"]["west"]["south"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["right"]["west"]["west"] = (th: TurboHearts) => th.leftPlayer.limboCards;

limboSources["across"] = {};
limboSources["across"]["north"] = {};
limboSources["across"]["north"]["north"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["across"]["north"]["east"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["across"]["north"]["south"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["across"]["north"]["west"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["across"]["east"] = {};
limboSources["across"]["east"]["north"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["across"]["east"]["east"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["across"]["east"]["south"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["across"]["east"]["west"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["across"]["south"] = {};
limboSources["across"]["south"]["north"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["across"]["south"]["east"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["across"]["south"]["south"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["across"]["south"]["west"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["across"]["west"] = {};
limboSources["across"]["west"]["north"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["across"]["west"]["east"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["across"]["west"]["south"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["across"]["west"]["west"] = (th: TurboHearts) => th.topPlayer.limboCards;

export class ReceivePassEvent implements Event {
  public type = "recv_pass" as const;

  private finished = false;

  constructor(private th: TurboHearts, private event: ReceivePassEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.to)(this.th);
    const cards = player.cards;
    this.updateCards(cards);
    let i = Z_HAND_CARDS;
    for (const card of cards) {
      card.sprite.zIndex = i++;
    }
    animateHand(this.th, this.event.to).then(() => {
      // TODO: this is resulting in jarring card flip
      this.th.app.stage.sortChildren();
      this.finished = true;
    });
  }

  private updateCards(hand: SpriteCard[]) {
    const limboSource = limboSources[this.th.pass][this.th.bottomSeat][this.event.to](this.th);
    const received = [...this.event.cards];
    while (limboSource.length > 0) {
      // Note: this is mutating both hand and limbo arrays
      const fromLimbo = limboSource.pop();
      if (fromLimbo.card === "BACK" && received.length > 0) {
        fromLimbo.card = received.pop();
        fromLimbo.sprite.texture = this.th.app.loader.resources[fromLimbo.card].texture;
        fromLimbo.hidden = false;
      } else if (fromLimbo.card !== "BACK" && received.length === 0) {
        // Passing known cards into another hand
        fromLimbo.card = "BACK";
        fromLimbo.sprite.texture = this.th.app.loader.resources["BACK"].texture;
        fromLimbo.hidden = true;
      }
      hand.push(fromLimbo);
    }
    sortSpriteCards(hand);
  }

  public isFinished() {
    return this.finished;
  }
}
