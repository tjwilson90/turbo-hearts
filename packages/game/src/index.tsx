if ( typeof process === 'undefined' ) {
  if ( typeof window === 'undefined' ) {
    throw new Error('Ooops ...');
  }
  (window as any).process = { 'env': {} };
}

import * as cookie from "cookie";
import * as React from "react";
import * as ReactDOM from "react-dom";
import { Provider } from "react-redux";
import { ChatInput } from "./chat/ChatInput";
import { Snapshotter } from "./game/snapshotter";
import { TurboHeartsEventSource } from "./game/TurboHeartsEventSource";
import { createGameAppStore } from "./state/createStore";
import { ChatEvent } from "./types";
import { GameApp } from "./ui/GameApp";
import { TurboHeartsStage } from "./view/TurboHeartsStage";

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

  const chatLog = document.getElementById("chat-log")!;
  const chatAppender = async (message: ChatEvent) => {
    // TODO: fix race
    console.log(message);
    const users = await store.getState().context.service.getUsers([message.userId]);
    const div = document.createElement("div");
    div.classList.add("chat-message-container");
    const nameSpan = document.createElement("span");
    nameSpan.classList.add("chat-user");
    nameSpan.textContent = users[message.userId];
    div.appendChild(nameSpan);
    const messageSpan = document.createElement("span");
    messageSpan.classList.add("chat-message");
    messageSpan.textContent = message.message;
    div.appendChild(messageSpan);
    chatLog.appendChild(div);
    div.scrollIntoView();
  };

  const eventSource = new TurboHeartsEventSource(gameId);
  eventSource.on("chat", chatAppender);

  const snapshotter = new Snapshotter(userId);
  eventSource.on("event", snapshotter.onEvent);
  snapshotter.on("snapshot", e => console.log(e));

  function start() {
    eventSource.connect();
  }

  const animator = new TurboHeartsStage(
    document.getElementById("turbo-hearts") as HTMLCanvasElement,
    userId,
    store.getState().context.service,
    start
  );
  snapshotter.on("snapshot", animator.acceptSnapshot);
  eventSource.once("end_replay", animator.endReplay);
  new ChatInput(document.getElementById("chat-input") as HTMLTextAreaElement, store.getState().context.service);

  ReactDOM.render(
    <Provider store={store}>
      <GameApp />
    </Provider>,
    document.getElementById("app-container")!
  );
});

