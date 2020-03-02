import {
    ChatEvent,
    EnterLobbyEvent,
    ExitLobbyEvent, FinishGameEvent, HumanLobbyPlayer,
    JoinGameEvent,
    LeaveGameEvent,
    LobbyPlayer,
    LobbyStateEvent,
    NewGameEvent
} from "./types";
import EventEmitter from "eventemitter3";
import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";

export interface LobbyGame {
    players: LobbyPlayer[];
    createdByUserId: string;
    updatedAt: Date;
    createdAt: Date;
}

export interface LobbySnapshot {
    games: {[gameId: string]: LobbyGame}
    subscriberUserIds: string[];
}

export class LobbySnapshotter {
    private emitter = new EventEmitter();
    
    public snapshot: LobbySnapshot = {
        games: {},
        subscriberUserIds: [],
    };

    public constructor(eventSource: TurboHeartsLobbyEventSource) {
        eventSource.on("lobby_state", this.onLobbyStateEvent);
        eventSource.on("enter", this.onEnterLobbyEvent);
        eventSource.on("exit", this.onExitLobbyEvent);
        eventSource.on("new_game", this.onNewGameEvent);
        eventSource.on("join_game", this.onJoinGameEvent);
        eventSource.on("leave_game", this.onLeaveGameEvent);
        eventSource.on("finish_game", this.onFinishGameEvent);
        eventSource.on("chat", this.onChatEvent);
    }

    public on(
        event: "snapshot",
        fn: (event: LobbySnapshot) => void
    ) {
        this.emitter.on("snapshot", fn);
    }

    public off(
        event: "snapshot",
        fn: (event: LobbySnapshot) => void
    ) {
        this.emitter.off("snapshot", fn);
    }

    private onLobbyStateEvent = (event: LobbyStateEvent) => {
        const games: {[gameId: string]: LobbyGame} = {};
        for (const gameId in event.games) {
            const game = event.games[gameId];
            games[gameId] = {
                createdAt: game.createdAt,
                updatedAt: game.createdAt,
                createdByUserId: game.userId,
                players: game.players
            }
        }

        this.snapshot = {
            games,
            subscriberUserIds: event.subscribers,
        }
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onEnterLobbyEvent = (event: EnterLobbyEvent) => {
        this.snapshot = {
            ...this.snapshot,
            subscriberUserIds: [...this.snapshot.subscriberUserIds, event.userId],
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onExitLobbyEvent = (event: ExitLobbyEvent) => {
        this.snapshot = {
            ...this.snapshot,
            subscriberUserIds: this.snapshot.subscriberUserIds.filter(userId => userId !== event.userId),
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onNewGameEvent = (event: NewGameEvent) => {
        this.snapshot = {
            ...this.snapshot,
            games: {
                ...this.snapshot.games,
                [event.gameId]: {
                    players: [{
                        type: "human",
                        userId: event.userId,
                    }],
                    createdAt: new Date(),
                    updatedAt: new Date(),
                    createdByUserId: event.userId,
                }
            }
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onJoinGameEvent = (event: JoinGameEvent) => {
        this.snapshot = {
            ...this.snapshot,
            games: {
                ...this.snapshot.games,
                [event.gameId]: {
                    ...this.snapshot.games[event.gameId],
                    players: this.snapshot.games[event.gameId].players.concat([{
                        type: "human",
                        userId: event.userId,
                    }]),
                }
            }
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onLeaveGameEvent = (event: LeaveGameEvent) => {
        this.snapshot = {
            ...this.snapshot,
            games: {
                ...this.snapshot.games,
                [event.gameId]: {
                    ...this.snapshot.games[event.gameId],
                    players: this.snapshot.games[event.gameId].players.filter(user => user.type === "bot" || event.userId !== event.userId)
                }
            }
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onFinishGameEvent = (event: FinishGameEvent) => {
        const games = {
            ...this.snapshot.games,
        };
        delete games[event.gameId];
        this.snapshot = {
            ...this.snapshot,
            games,
        };
        this.emitter.emit("snapshot", this.snapshot);
    }
    private onChatEvent = (_event: ChatEvent) => {
        // TODO
    }

}
