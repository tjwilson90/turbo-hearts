import * as cookie from "cookie";
import { ChatInput } from "./chat/ChatInput";
import { TurboHeartsService } from "./game/TurboHeartsService";
import { Snapshotter } from "./game/snapshotter";
import { TurboHeartsEventSource } from "./game/TurboHeartsEventSource";
import { ChatEvent } from "./types";
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
  const service = new TurboHeartsService(gameId);
  const chatLog = document.getElementById("chat-log")!;
  const chatAppender = async (message: ChatEvent) => {
    // TODO: fix race
    console.log(message);
    const users = await service.getUsers([message.userId]);
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
  eventSource.onAny(snapshotter.onEvent);
  snapshotter.on("snapshot", e => console.log(e));

  function start() {
    eventSource.connect();
  }

  const animator = new TurboHeartsStage(
    document.getElementById("turbo-hearts") as HTMLCanvasElement,
    userId,
    service,
    start
  );
  snapshotter.on("snapshot", animator.acceptSnapshot);
  eventSource.once("end_replay", animator.endReplay);
  new ChatInput(document.getElementById("chat-input") as HTMLTextAreaElement, service);
});

