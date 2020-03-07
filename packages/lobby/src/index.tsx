if (typeof process === "undefined") {
  if (typeof window === "undefined") {
    throw new Error("Ooops ...");
  }
  (window as any).process = { env: {} };
}

import { createStore } from "./state/createStore";
import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";
import { LobbySubscriber } from "./lobbySubscriber";
import { TurboHeartsLobbyService } from "./TurboHeartsLobbyService";
import * as ReactDOM from "react-dom";
import * as React from "react";
import { Lobby } from "./components/Lobby";
import { Provider } from "react-redux";

document.addEventListener("DOMContentLoaded", () => {
  const lobbyEventSource = new TurboHeartsLobbyEventSource();
  const service = new TurboHeartsLobbyService();

  const store = createStore();

  new LobbySubscriber(lobbyEventSource, service, store);

  ReactDOM.render(
      <Provider store={store}>
        <Lobby service={service}/>
      </Provider>,
      document.getElementById("lobby")
  )

  lobbyEventSource.connect();
});
