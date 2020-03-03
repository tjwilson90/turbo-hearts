import { EventEmitter } from "eventemitter3";
import { LobbyEvent } from "./types";

function renameProp(object: any, from: string, to: string, remap?: (a: any) => any) {
    object[to] = remap ? remap(object[from]) : object[from];
    delete object[from];
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
    renameProp(event, "last_updated_time", "updatedAt", time => new Date(time) );
    }
    if (event.hasOwnProperty("created_time")) {
        renameProp(event, "created_time", "createdAt", time => new Date(time));
    }
}

function unrustify(event: any): LobbyEvent {
    renameCommon(event);

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
            return event;
        case "join_game":
            event.userId = event.player.player.user_id;
            return event;
        case "new_game":
            event.createdBy = event.game.created_by;
            return event;
        case "leave_game":
        case "finish_game":
        case "chat":
        default:
            return event;
    }
}

export class TurboHeartsLobbyEventSource {
    private eventSource: EventSource | undefined;
    private emitter = new EventEmitter();

    public connect() {
        this.eventSource = new EventSource(`/lobby/subscribe`);
        this.eventSource.addEventListener("message", this.handleEvent);
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
}
