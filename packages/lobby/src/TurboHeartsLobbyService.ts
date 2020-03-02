import { BotAlgorithms, Rules } from "./types";

export class TurboHeartsLobbyService {
    private userNames: { [key: string]: string } = {};

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

    public createLobby(rules: Rules) {
        return fetch(`/lobby/new`, this.requestWithBody({ rules }));
    }

    public leaveLobby(gameId: string) {
        return fetch(`/lobby/leave`, this.requestWithBody({ game_id: gameId }));
    }

    public joinLobby(gameId: string, rules: Rules) {
        return fetch(`/lobby/join`, this.requestWithBody({ game_id: gameId, rules }));
    }

    public addBot(gameId: string, rules: Rules, algorithm: BotAlgorithms) {
        return fetch(`/lobby/add_bot`, this.requestWithBody({ game_id: gameId, rules, algorithm }));
    }

    public chat(message: string) {
        return fetch(`/lobby/chat`, this.requestWithBody({ message }));
    }

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
