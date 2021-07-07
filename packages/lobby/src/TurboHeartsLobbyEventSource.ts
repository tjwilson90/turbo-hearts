import { EventEmitter } from "eventemitter3";
import { LobbyEvent } from "./types";

function renameProp(object: any, from: string, to: string, remap?: (a: any) => any) {
    object[to] = remap ? remap(object[from]) : object[from];
    if (from !== to) {
        delete object[from];
    }
}

function renameCommon(event: any) {
    if (event.hasOwnProperty("user_id")) {
        renameProp(event, "user_id", "userId");
    }
    if (event.hasOwnProperty("game_id")) {
        renameProp(event, "game_id", "gameId");
    }
    if (event.hasOwnProperty("created_by")) {
        renameProp(event, "created_by", "createdBy");
    }
    if (event.hasOwnProperty("last_updated_time")) {
        renameProp(event, "last_updated_time", "updatedAt", time => time == null ? undefined : new Date(time) );
    }
    if (event.hasOwnProperty("created_time")) {
        renameProp(event, "created_time", "createdAt", time => time == null ? undefined : new Date(time));
    }
    if (event.hasOwnProperty("started_time")) {
        renameProp(event, "started_time", "startedAt", time => time == null ? undefined : new Date(time));
    }
    if (event.hasOwnProperty("timestamp")) {
        renameProp(event, "timestamp", "timestamp", time => time == null ? undefined : new Date(time));
    }
}

function unrustify(event: any): LobbyEvent {
    renameCommon(event);

    console.log(event);

    switch (event.type) {
        case "join_lobby":
            event.type = "enter";
            return event;
        case "leave_lobby":
            event.type = "exit";
            return event;
        case "lobby_state":
            for (const gameId in event.games) {
                const game = event.games[gameId];
                game.gameId = gameId;
                renameCommon(game);
                game.players = game.players.map((player: any) => {
                    renameCommon(player.player);
                    return player.player;
                });
                delete game.last_updated_by;
                delete game.seed;
            }
            for (const chat of event.chat) {
                renameCommon(chat);
            }
            return event;
        case "join_game":
            renameCommon(event.player.player);
            event.player = event.player.player;
            return event;
        case "new_game":
            event.createdBy = event.player.player.user_id;
            return event;
        case "start_game":
            renameCommon(event.north);
            renameCommon(event.west);
            renameCommon(event.south);
            renameCommon(event.east);
            return event;
        case "leave_game":
            renameCommon(event.player);
            return event;
        case "finish_game":
        case "chat":
        default:
            return event;
    }
}

export class TurboHeartsLobbyEventSource {
    private eventSource: EventSource | undefined;
    private emitter = new EventEmitter();
    private connectionOpenedSuccessfully = false;

    public connect() {
        this.eventSource = new EventSource(`/lobby/subscribe`);
        this.eventSource.addEventListener("message", this.handleEvent);
        this.eventSource.addEventListener("error", this.handleDisconnect);
        this.eventSource.addEventListener("open", this.handleOpen);
    }

    public on<K extends LobbyEvent>(event: K["type"], fn: (event: K) => void) {
        this.emitter.on(event, fn);
    }

    public onAny(fn: (event: LobbyEvent) => void) {
        this.emitter.on("event", fn);
    }

    public off<K extends LobbyEvent>(event: K["type"], fn: (event: K) => void) {
        this.emitter.off(event, fn);
    }

    public once<K extends LobbyEvent>(event: K["type"], fn: (event: K) => void) {
        this.emitter.once(event, fn);
    }

    private handleEvent = (event: MessageEvent) => {
        const parsedEvent = JSON.parse(event.data);
        console.log(parsedEvent);
        const rawEvent: LobbyEvent = unrustify(parsedEvent);
        this.emitter.emit("event", rawEvent);
        this.emitter.emit(rawEvent.type, rawEvent);
    };

    private handleOpen = () => {
        this.connectionOpenedSuccessfully = true;
    }

    private handleDisconnect = () => {
        if (!this.connectionOpenedSuccessfully) {
            document.cookie = "AUTH_TOKEN=; expires = Thu, 01 Jan 1970 00:00:00 GMT";
            document.cookie = "USER_ID=; expires = Thu, 01 Jan 1970 00:00:00 GMT";
            document.cookie = "USER_NAME=; expires = Thu, 01 Jan 1970 00:00:00 GMT";
            location.reload();
        } else {
            this.eventSource!.close();
            this.connectionOpenedSuccessfully = false;
            setTimeout(this.connect, 1000);
        }
    }
}
