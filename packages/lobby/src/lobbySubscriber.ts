import {
    ChatEvent,
    EnterLobbyEvent,
    ExitLobbyEvent,
    FinishGameEvent,
    JoinGameEvent,
    LeaveGameEvent,
    LobbyStateEvent,
    NewGameEvent, StartGameEvent
} from "./types";
import { Store } from "redux";
import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";
import { LobbyState } from "./state/types";
import { AppendChat, DeleteLobbyGame, UpdateChatUserIds, UpdateLobbyGame, UpdateUserNames } from "./state/actions";
import { TurboHeartsLobbyService } from "./TurboHeartsLobbyService";

export class LobbySubscriber {
    public constructor(eventSource: TurboHeartsLobbyEventSource,
                       private lobbyService: TurboHeartsLobbyService,
                       private store: Store<LobbyState>) {
        eventSource.on("lobby_state", this.onLobbyStateEvent);
        eventSource.on("enter", this.onEnterLobbyEvent);
        eventSource.on("exit", this.onExitLobbyEvent);
        eventSource.on("new_game", this.onNewGameEvent);
        eventSource.on("join_game", this.onJoinGameEvent);
        eventSource.on("leave_game", this.onLeaveGameEvent);
        eventSource.on("finish_game", this.onFinishGameEvent);
        eventSource.on("start_game", this.onStartGame);
        eventSource.on("chat", this.onChatEvent);
    }

    private onLobbyStateEvent = (event: LobbyStateEvent) => {
        const collectedUserIds = new Set<string>();

        for (const gameId in event.games) {
            const game = event.games[gameId];

            this.store.dispatch(UpdateLobbyGame({
                gameId,
                lobbyGame:  {
                    gameId,
                    createdAt: game.createdAt,
                    updatedAt: game.updatedAt,
                    createdBy: game.createdBy,
                    players: game.players,
                    startedAt: game.startedAt,
                },
            }))

            collectedUserIds.add(game.createdBy);

            const humanPlayers = game.players.filter(player => player.type === "human");
            for (const player of humanPlayers) {
                collectedUserIds.add(player.userId);
            }
        }

        for (const chat of event.chat) {
            this.store.dispatch(AppendChat({
                room: "lobby",
                message: {
                    date: chat.timestamp,
                    userId: chat.userId,
                    message: chat.message,
                    generated: false,
                    substitutions: [],
                }
            }));
            collectedUserIds.add(chat.userId);
        }

        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: event.subscribers,
        }));

        for (const userId of event.subscribers) {
            collectedUserIds.add(userId);
        }

        for (const userId of collectedUserIds) {
            this.maybeLoadUserId(userId);
        }

        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: this.store.getState().users.ownUserId,
                message: "joined.",
                generated: true,
                substitutions: [],
            }
        }));
    }
    private onEnterLobbyEvent = (event: EnterLobbyEvent) => {
        const subscriberUserIds = this.store.getState().chats.lobby.userIds;
        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: [...subscriberUserIds, event.userId],
        }));
        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: event.userId,
                message: "joined.",
                generated: true,
                substitutions: [],
            }
        }));
        this.maybeLoadUserId(event.userId);
    }
    private onExitLobbyEvent = (event: ExitLobbyEvent) => {
        const subscriberUserIds = this.store.getState().chats.lobby.userIds;
        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: subscriberUserIds.filter(userId => userId !== event.userId),
        }));
        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: event.userId,
                message: "quit.",
                generated: true,
                substitutions: [],
            }
        }));
        this.maybeLoadUserId(event.userId);
    }
    private onNewGameEvent = (event: NewGameEvent) => {
        this.store.dispatch(UpdateLobbyGame({
            gameId: event.gameId,
            lobbyGame:  {
                gameId: event.gameId,
                players: [{
                    type: "human",
                    userId: event.createdBy,
                }],
                createdAt: new Date(),
                updatedAt: new Date(),
                createdBy: event.createdBy,
                startedAt: undefined,
            }
        }));
        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: event.createdBy,
                message: "created a game.",
                generated: true,
                substitutions: [],
            }
        }));
        this.maybeLoadUserId(event.createdBy);
    }
    private onJoinGameEvent = (event: JoinGameEvent) => {
        const game = this.store.getState().games[event.gameId];
        this.store.dispatch(UpdateLobbyGame({
            gameId: event.gameId,
            lobbyGame:  {
                ...game,
                players: game.players.concat([event.player]),
            }
        }));
        if (event.player.type === "human") {
            this.maybeLoadUserId(event.player.userId);
        }
    }
    private onStartGame = (event: StartGameEvent) => {
        const game = this.store.getState().games[event.gameId];
        this.store.dispatch(UpdateLobbyGame({
            gameId: event.gameId,
            lobbyGame:  {
                ...game,
                startedAt: new Date(),
            }
        }));
        for (const player of [event.north, event.east, event.south, event.west]) {
            this.maybeLoadUserId(player);
        }
        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: undefined,
                message: "Game started between $0, $1, $2, and $3! $4",
                generated: true,
                substitutions: [
                    { type: "user", userId: event.north },
                    { type: "user", userId: event.east },
                    { type: "user", userId: event.south },
                    { type: "user", userId: event.west },
                    { type: "game", gameId: event.gameId },
                ],
            }
        }));
    }
    private onLeaveGameEvent = (event: LeaveGameEvent) => {
        const game = this.store.getState().games[event.gameId];
        const players = game.players.filter(user => user.type === "bot" || event.userId !== event.userId);
        if (players.length === 0) {
            this.store.dispatch(DeleteLobbyGame({
                gameId: event.gameId,
            }));
        } else {
            this.store.dispatch(UpdateLobbyGame({
                gameId: event.gameId,
                lobbyGame:  {
                    ...game,
                    players
                }
            }));
        }
    }
    private onFinishGameEvent = (event: FinishGameEvent) => {
        this.store.dispatch(DeleteLobbyGame({
            gameId: event.gameId,
        }));
    }
    private onChatEvent = (event: ChatEvent) => {
        this.store.dispatch(AppendChat({
            room: "lobby",
            message: {
                date: new Date(),
                userId: event.userId,
                message: event.message,
                generated: false,
                substitutions: [],
            }
        }));
        this.maybeLoadUserId(event.userId);
    }

    private async maybeLoadUserId(userId: string) {
        if (this.store.getState().users.userNamesByUserId[userId] === undefined) {
            const userName = await this.lobbyService.getUser(userId);
            this.store.dispatch(UpdateUserNames({
                ...this.store.getState().users.userNamesByUserId,
                [userId]: userName,
            }));
        }
    }
}
