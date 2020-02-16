import { TurboHearts } from "../game/TurboHearts";
import { Event, ReceivePassEventData, SpriteCard } from "../types";
import { animateHand } from "./animations/animations";
import { getPlayerAccessor } from "./playerAccessors";
import { sortSpriteCards } from "../game/sortCards";

const limboSources: {
  [pass: string]: {
    [bottomSeat: string]: {
      [passFrom: string]: (th: TurboHearts) => SpriteCard[];
    };
  };
} = {};
limboSources["Left"] = {};
limboSources["Left"]["north"] = {};
limboSources["Left"]["north"]["north"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["north"]["east"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["north"]["south"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["north"]["west"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["east"] = {};
limboSources["Left"]["east"]["north"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["east"]["east"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["east"]["south"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["east"]["west"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["south"] = {};
limboSources["Left"]["south"]["north"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["south"]["east"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["south"]["south"] = (th: TurboHearts) => th.rightPlayer.limboCards;
limboSources["Left"]["south"]["west"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["west"] = {};
limboSources["Left"]["west"]["north"] = (th: TurboHearts) => th.bottomPlayer.limboCards;
limboSources["Left"]["west"]["east"] = (th: TurboHearts) => th.leftPlayer.limboCards;
limboSources["Left"]["west"]["south"] = (th: TurboHearts) => th.topPlayer.limboCards;
limboSources["Left"]["west"]["west"] = (th: TurboHearts) => th.rightPlayer.limboCards;

export class ReceivePassEvent implements Event {
  private finished = false;

  constructor(private th: TurboHearts, private event: ReceivePassEventData) {}

  public begin() {
    const player = getPlayerAccessor(this.th.bottomSeat, this.event.to)(this.th);
    const cards = player.cards;
    this.updateCards(cards);
    let i = 100;
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
