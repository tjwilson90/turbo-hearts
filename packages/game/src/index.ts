import * as cookie from "cookie";
import { PlaySubmitter } from "./game/PlaySubmitter";
import { TurboHearts } from "./game/TurboHearts";
import { TurboHeartsEventSource } from "./game/TurboHeartsEventSource";
import "./styles/style.scss";
import { ChatInput } from "./chat/ChatInput";
import { ChatEvent } from "./types";

document.addEventListener("DOMContentLoaded", () => {
  const userId = cookie.parse(document.cookie)["NAME"];
  if (userId?.length === 0) {
    document.body.innerHTML = "Missing user id";
    return;
  }
  const gameId = window.location.hash.substring(1);
  if (gameId.length !== 36) {
    document.body.innerHTML = "Missing game id";
    return;
  }
  const submitter = new PlaySubmitter(gameId);
  const th = new TurboHearts(document.getElementById("turbo-hearts") as HTMLCanvasElement, userId, submitter);
  (window as any).th = th;
  const chatLog = document.getElementById("chat-log")!;
  const chatAppender = (message: ChatEvent) => {
    const div = document.createElement("div");
    div.classList.add("chat-message-container");
    const nameSpan = document.createElement("span");
    nameSpan.classList.add("chat-user");
    nameSpan.textContent = message.name;
    div.appendChild(nameSpan);
    const messageSpan = document.createElement("span");
    messageSpan.classList.add("chat-message");
    messageSpan.textContent = message.message;
    div.appendChild(messageSpan);
    chatLog.appendChild(div);
    div.scrollIntoView();
  };
  new TurboHeartsEventSource(th, gameId, chatAppender);

  new ChatInput(document.getElementById("chat-input") as HTMLTextAreaElement, gameId);
});
