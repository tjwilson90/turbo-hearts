import { Card } from "../types";

export class PlaySubmitter {
  constructor(private gameId: string) {}

  private requestWithBody(body: any): RequestInit {
    return {
      credentials: "include",
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(body)
    };
  }

  public passCards = (cards: Card[]) => {
    return fetch(`/game/pass`, this.requestWithBody({ game_id: this.gameId, cards }));
  };

  public chargeCards = (cards: Card[]) => {
    return fetch(`/game/charge`, this.requestWithBody({ game_id: this.gameId, cards }));
  };

  public playCard = (card: Card) => {
    return fetch(`/game/play`, this.requestWithBody({ game_id: this.gameId, card }));
  };
}
