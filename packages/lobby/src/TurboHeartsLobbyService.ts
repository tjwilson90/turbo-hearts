import { BotStrategy, Rules } from "./types";

export class TurboHeartsLobbyService {
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

    public addBot(gameId: string, rules: Rules, strategy: BotStrategy) {
        return fetch(`/lobby/add_bot`, this.requestWithBody({ game_id: gameId, rules, strategy }));
    }

    public startGame(gameId: string) {
        return fetch(`/lobby/start`, this.requestWithBody({ game_id: gameId }));
    }

    public chat(message: string) {
        return fetch(`/lobby/chat`, this.requestWithBody({ message }));
    }

    public getUser = async (userId: string) => {
        const resp = await fetch(`/users`, this.requestWithBody({ ids: [userId] }));
        const json = (await resp.json()) as { name: string; id: string }[];
        for (const item of json) {
            return item.name;
        }
        return "Unknown";
    };
}
