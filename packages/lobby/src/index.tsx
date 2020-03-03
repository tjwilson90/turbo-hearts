if (typeof process === "undefined") {
  if (typeof window === "undefined") {
    throw new Error("Ooops ...");
  }
  (window as any).process = { env: {} };
}


import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";
import { LobbySnapshot, LobbySnapshotter } from "./lobbySnapshotter";
import { TurboHeartsLobbyService } from "./TurboHeartsLobbyService";
import * as ReactDOM  from "react-dom";
import * as React from "react";
import { Lobby } from "./components/Lobby";

function render(snapshot: LobbySnapshot, service: TurboHeartsLobbyService) {
  ReactDOM.render(
      <Lobby {...snapshot} messages={[]} service={service}/>,
      document.getElementById("lobby")
  )
}

document.addEventListener("DOMContentLoaded", () => {
  const eventSource = new TurboHeartsLobbyEventSource();
  const service = new TurboHeartsLobbyService();

  const snaphotter = new LobbySnapshotter(eventSource)
  snaphotter.on("snapshot", (snapshot) => render(snapshot, service));

  render(snaphotter.snapshot, service);

  eventSource.connect();
  
  service.createLobby("classic");
  service.chat("Hello world");
});
