import {
    ChatEvent,
    EnterLobbyEvent,
    ExitLobbyEvent,
    FinishGameEvent,
    JoinGameEvent,
    LeaveGameEvent,
    LobbyStateEvent,
    NewGameEvent
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
        eventSource.on("chat", this.onChatEvent);
    }

    private onLobbyStateEvent = (event: LobbyStateEvent) => {
        for (const gameId in event.games) {
            const game = event.games[gameId];

            this.store.dispatch(UpdateLobbyGame({
                gameId,
                lobbyGame:  {
                    gameId,
                    createdAt: game.createdAt,
                    updatedAt: game.updatedAt,
                    createdBy: game.createdBy,
                    players: game.players
                },
            }))
        }

        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: event.subscribers,
        }));

        for (const userId of event.subscribers) {
            this.maybeLoadUserId(userId);
        }
    }
    private onEnterLobbyEvent = (event: EnterLobbyEvent) => {
        const subscriberUserIds = this.store.getState().chats.lobby.userIds;
        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: [...subscriberUserIds, event.userId],
        }));
        this.maybeLoadUserId(event.userId);
    }
    private onExitLobbyEvent = (event: ExitLobbyEvent) => {
        const subscriberUserIds = this.store.getState().chats.lobby.userIds;
        this.store.dispatch(UpdateChatUserIds({
            room: "lobby",
            userIds: subscriberUserIds.filter(userId => userId !== event.userId),
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
            }
        }));
        this.maybeLoadUserId(event.userId);
    }

    private async maybeLoadUserId(userId: string) {
        const userName = await this.lobbyService.getUser(userId);
        this.store.dispatch(UpdateUserNames({
            ...this.store.getState().users.userNamesByUserId,
            [userId]: userName,
        }));
    }
}
