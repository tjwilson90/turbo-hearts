import { TurboHeartsLobbyEventSource } from "./TurboHeartsLobbyEventSource";
import { LobbySnapshot, LobbySnapshotter } from "./lobbySnapshotter";
import { TurboHeartsLobbyService } from "./TurboHeartsLobbyService";

function render(snapshot: LobbySnapshot, service: TurboHeartsLobbyService) {
  console.log(snapshot);
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
