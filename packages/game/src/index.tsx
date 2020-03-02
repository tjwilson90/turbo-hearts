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
import { SitEventData, ChatEvent, Seat } from "./types";
import { AppendChat, UpdateActions } from "./state/actions";
import { getBottomSeat } from "./view/TurboHeartsStage";

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
  const ctx = store.getState().context;
  const userDispatcher = new UserDispatcher(ctx.service, userId, store.dispatch);
  ctx.eventSource.once("sit", (event: SitEventData) => {
    // console.log(event);
    userDispatcher.loadUsersForGame(event);
  });
  ctx.eventSource.on("chat", (chat: ChatEvent) => {
    userDispatcher.loadUsers([chat.userId]);
    store.dispatch(AppendChat(chat));
  });
  ctx.snapshotter.on("snapshot", snapshot => {
    // console.log(snapshot);
    const bottomSeat = getBottomSeat(snapshot.next, userId);
    const seatOrderForBottomSeat: { [bottomSeat in Seat]: Seat[] } = {
      north: ["south", "west", "north", "east"],
      east: ["west", "north", "east", "south"],
      south: ["north", "east", "south", "west"],
      west: ["east", "south", "west", "north"]
    };
    const actions = {
      top: snapshot.next[seatOrderForBottomSeat[bottomSeat][0]].action,
      right: snapshot.next[seatOrderForBottomSeat[bottomSeat][1]].action,
      bottom: snapshot.next[seatOrderForBottomSeat[bottomSeat][2]].action,
      left: snapshot.next[seatOrderForBottomSeat[bottomSeat][3]].action
    };
    store.dispatch(UpdateActions(actions));
  });
  ctx.trickTracker.on("trick", trick => {
    console.log(trick);
  });
  ReactDOM.render(
    <Provider store={store}>
      <GameApp userDispatcher={userDispatcher} />
    </Provider>,
    document.getElementById("app-container")!
  );
});
