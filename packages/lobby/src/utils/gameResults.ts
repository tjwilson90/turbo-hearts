import { GameResult } from "../types";

export interface ScoreDelta {
    score: number;
    delta: number;
}

export type HandScores = { [key: string]: ScoreDelta };

export interface GameScores {
    userIds: string[];
    totals: HandScores;
    scores: HandScores[];
}

export interface LeaderboardEntry {
    userId: string;
    games: number;
    points: number;
}

export function calculateScores(game: GameResult): GameScores {
    const handScores = [];
    for (const hand of game.hands) {
        const chargeSet = new Set([].concat(...hand.charges));
        const qsCharged = chargeSet.has("QS");
        const jdCharged = chargeSet.has("JD");
        const tcCharged = chargeSet.has("TC");
        const ahCharged = chargeSet.has("AH");
        const handResult: HandScores = {};
        const scores = [];
        for (let i = 0; i < 4; i++) {
            const player = game.players[i];
            const hearts = hand.hearts[i];
            let score = hearts * (ahCharged ? 2 : 1);
            const qs = hand.qsWinnerId === player.userId;
            if (qs) {
                score += 13 * (qsCharged ? 2 : 1);
            }
            if (hearts === 13 && qs) {
                score *= -1;
            }
            const jd = hand.jdWinnerId === player.userId;
            if (jd) {
                score += -10 * (jdCharged ? 2 : 1);
            }
            const tc = hand.tcWinnerId === player.userId;
            if (tc) {
                score *= tcCharged ? 4 : 2;
            }
            scores.push(score);
            handResult[player.userId] = { score, delta: 0 };
        }
        const deltas = scoresToDelta(scores);
        for (let i = 0; i < 4; i++) {
            const player = game.players[i];
            handResult[player.userId].delta = deltas[i];
        }
        handScores.push(handResult);
    }
    const totals: { [key: string]: ScoreDelta } = {};
    for (let i = 0; i < 4; i++) {
        const player = game.players[i];
        totals[player.userId] = { score: 0, delta: 0 };
    }
    for (const hand of handScores) {
        for (let i = 0; i < 4; i++) {
            const player = game.players[i];
            totals[player.userId].score += hand[player.userId].score;
            totals[player.userId].delta += hand[player.userId].delta;
        }
    }
    const finalResult = { totals, scores: handScores, userIds: game.players.map(p => p.userId) };
    return finalResult;
}

export function calculateLeaderboard(scores: GameScores[]) {
    const leaderboard = new Map<string, LeaderboardEntry>();
    for (const score of scores) {
        for (const userId of score.userIds) {
            if (!leaderboard.has(userId)) {
                leaderboard.set(userId, {
                    userId,
                    games: 0,
                    points: 0
                });
            }
            const totals = score.totals[userId];
            const entry = leaderboard.get(userId)!;
            entry.games++;
            entry.points += totals.delta;
        }
    }
    return Array.from(leaderboard.values());
}

export function scoresToDelta(scores: number[]): number[] {
    return [
        scores[1] + scores[2] + scores[3] - 3 * scores[0],
        scores[0] + scores[2] + scores[3] - 3 * scores[1],
        scores[0] + scores[1] + scores[3] - 3 * scores[2],
        scores[0] + scores[1] + scores[2] - 3 * scores[3]
    ];
}
