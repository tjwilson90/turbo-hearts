if (typeof process === "undefined") {
  if (typeof window === "undefined") {
    throw new Error("Ooops ...");
  }
  (window as any).process = { env: {} };
}

import * as cookie from "cookie";
import * as React from "react";
import * as ReactDOM from "react-dom";
import { Provider } from "react-redux";
import { createGameAppStore } from "./state/createStore";
import { UserDispatcher } from "./state/UserDispatcher";
import { GameApp } from "./ui/GameApp";
import { SitEventData } from "./types";

document.addEventListener("DOMContentLoaded", () => {
  const userId = cookie.parse(document.cookie)["USER_ID"];
  if (userId?.length === 0) {
    document.body.innerHTML = "Missing user id";
    return;
  }
  const gameId = window.location.hash.substring(1);
  if (gameId.length !== 36) {
    document.body.innerHTML = "Missing game id";
    return;
  }

  const store = createGameAppStore(gameId);
  const userDispatcher = new UserDispatcher(store.getState().context.service, userId, store.dispatch);
  store.getState().context.eventSource.once("sit", event => {
    userDispatcher.loadUsersForGame(event as SitEventData);
  });
  ReactDOM.render(
    <Provider store={store}>
      <GameApp userDispatcher={userDispatcher} />
    </Provider>,
    document.getElementById("app-container")!
  );
});
