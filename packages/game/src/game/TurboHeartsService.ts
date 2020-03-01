import { Card } from "../types";

export class TurboHeartsService {
  private userNames: { [key: string]: string } = {};
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

  public chat = (message: string) => {
    return fetch(`/game/chat`, this.requestWithBody({ game_id: this.gameId, message }));
  };

  public getUsers = async (userIds: string[]) => {
    const result: { [key: string]: string } = {};
    const toRequest: string[] = [];
    for (const userId of userIds) {
      if (this.userNames[userId] !== undefined) {
        result[userId] = this.userNames[userId];
      } else {
        toRequest.push(userId);
      }
    }
    if (toRequest.length === 0) {
      return result;
    }
    const resp = await fetch(`/users`, this.requestWithBody({ ids: toRequest }));
    const json = (await resp.json()) as { name: string; id: string }[];
    for (const item of json) {
      result[item.id] = item.name;
      this.userNames[item.id] = item.name;
    }
    return result;
  };
}
