import { BotStrategy, Rules, GameResult } from "./types";

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

    public addBot(gameId: string, rules: Rules, strategy: BotStrategy) {
        return fetch(`/lobby/add_bot`, this.requestWithBody({ game_id: gameId, rules, strategy }));
    }

    public startGame(gameId: string) {
        return fetch(`/lobby/start`, this.requestWithBody({ game_id: gameId }));
    }

    public chat(message: string) {
        return fetch(`/lobby/chat`, this.requestWithBody({ message }));
    }

    public getRecentGames() {
        return fetch(`/summary/leaderboard`, {
            credentials: "include",
            method: "GET",
            headers: {
                "Content-Type": "application/json"
            }
        })
            .then(resp => resp.json())
            .then((json: any[]) => {
                const results: GameResult[] = [];
                for (const game of json) {
                    results.push({
                        gameId: game.game_id,
                        time: game.completed_time,
                        players: game.players.map((player: any) => ({
                            userId: player.user_id
                        })),
                        hands: game.hands.map((hand: any) => ({
                            charges: hand.charges,
                            hearts: hand.hearts_won,
                            qsWinnerId: hand.queen_winner,
                            tcWinnerId: hand.ten_winner,
                            jdWinnerId: hand.jack_winner
                        }))
                    });
                }
                return results;
            });
    }

    public getUser = async (userId: string) => {
        return this.getUsers([userId]).then(users => users[userId]);
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
        for (const userId of userIds) {
            if (result[userId] === undefined) {
                result[userId] = "Unknown";
            }
        }
        return result;
    };
}
