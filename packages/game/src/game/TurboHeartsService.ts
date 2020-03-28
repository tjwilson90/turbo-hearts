import { Card, Seat } from "../types";
import { User } from "../state/types";

export class TurboHeartsService {
  private userNames: { [key: string]: User } = {};
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

  public claim = () => {
    return fetch(`/game/claim`, this.requestWithBody({ game_id: this.gameId }));
  };

  public acceptClaim = (claimer: Seat) => {
    return fetch(`/game/accept_claim`, this.requestWithBody({ game_id: this.gameId, claimer }));
  };

  public rejectClaim = (claimer: Seat) => {
    return fetch(`/game/reject_claim`, this.requestWithBody({ game_id: this.gameId, claimer }));
  };

  public getUsers = async (userIds: string[]) => {
    const result: { [key: string]: User } = {};
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
      const user = {
        userId: item.id,
        name: item.name
      };
      result[item.id] = user;
      this.userNames[item.id] = user;
    }
    return result;
  };
}
