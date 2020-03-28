import { createStore } from "./state/createStore";
import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";
import { LobbySubscriber } from "./lobbySubscriber";
import { TurboHeartsLobbyService } from "./TurboHeartsLobbyService";
import * as ReactDOM from "react-dom";
import * as React from "react";
import { Lobby } from "./components/Lobby";
import { Provider } from "react-redux";
import { SetLeagueGames, UpdateUserNames } from "./state/actions";

document.addEventListener("DOMContentLoaded", async () => {
    const lobbyEventSource = new TurboHeartsLobbyEventSource();
    const service = new TurboHeartsLobbyService();

    const store = createStore();

    new LobbySubscriber(lobbyEventSource, service, store);

    ReactDOM.render(
        <Provider store={store}>
            <Lobby service={service} />
        </Provider>,
        document.getElementById("lobby")
    );

    lobbyEventSource.connect();

    // TODO: move this somewhere nice
    const games = await service.getRecentGames();
    const gameUsers = new Set<string>();
    for (const game of games) {
        for (const player of game.players) {
            gameUsers.add(player.userId);
        }
    }
    store.dispatch(SetLeagueGames(games));
    const users = await service.getUsers(Array.from(gameUsers.values()));
    store.dispatch(
        UpdateUserNames({
            ...store.getState().users.userNamesByUserId,
            ...users
        })
    );
});
