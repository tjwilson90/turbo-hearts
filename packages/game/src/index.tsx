import * as cookie from "cookie";
import * as React from "react";
import * as ReactDOM from "react-dom";
import { Provider } from "react-redux";
import { createGameAppStore } from "./state/createStore";
import { UserDispatcher } from "./state/UserDispatcher";
import { GameApp } from "./ui/GameApp";
import {
  SitEventData,
  ChatEvent,
  Seat,
  HandCompleteEventData,
  ClaimEventData,
  AcceptClaimEventData,
  RejectClaimEventData,
  GAME_BOT,
  DealEventData,
  StartChargingEventData,
  StartTrickEventData,
  JoinGameEventData,
  LeaveGameEventData,
} from "./types";
import {
  AppendChat,
  UpdateActions,
  AppendTrick,
  ResetTricks,
  AppendHandScore,
  EnableSpectatorMode,
  SetLocalPass,
  UpdateClaims,
  ResetClaims
} from "./state/actions";
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
    if (
      userId !== event.north.userId &&
      userId !== event.east.userId &&
      userId !== event.south.userId &&
      userId !== event.west.userId
    ) {
      store.dispatch(EnableSpectatorMode());
    }
    userDispatcher.loadUsersForGame(event);
  });
  ctx.eventSource.on("deal", (deal: DealEventData) => {
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `Dealing "${deal.pass}" hand.`
      })
    );
  });
  ctx.eventSource.on("start_charging", (_charge: StartChargingEventData) => {
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `Beginning charge phase.`
      })
    );
  });
  ctx.eventSource.on("start_trick", (_startTrick: StartTrickEventData) => {
    if (store.getState().game.tricks.length === 0)
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `Beginning play.`
      })
    );
  });
  ctx.eventSource.on("chat", (chat: ChatEvent) => {
    userDispatcher.loadUsers([chat.userId]);
    store.dispatch(AppendChat(chat));
  });
  ctx.eventSource.on("join_game", (event: JoinGameEventData) => {
    userDispatcher.loadUsers([event.userId]);
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `__${event.userId} has joined the game.`
      })
    );
  });
  ctx.eventSource.on("leave_game", (event: LeaveGameEventData) => {
    userDispatcher.loadUsers([event.userId]);
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `__${event.userId} has left the game.`
      })
    );
  });
  ctx.eventSource.on("claim", (event: ClaimEventData) => {
    store.dispatch(UpdateClaims(event));
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `__${event.seat} has claimed the rest of the tricks.`
      })
    );
  });
  ctx.eventSource.on("accept_claim", (event: AcceptClaimEventData) => {
    store.dispatch(UpdateClaims(event));
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `__${event.acceptor} has accepted __${event.claimer}'s claim.`
      })
    );
  });
  ctx.eventSource.on("reject_claim", (event: RejectClaimEventData) => {
    store.dispatch(UpdateClaims(event));
    store.dispatch(
      AppendChat({
        userId: GAME_BOT,
        message: `__${event.rejector} has rejected __${event.claimer}'s claim.`
      })
    );
  });
  ctx.eventSource.on("hand_complete", (scores: HandCompleteEventData) => {
    store.dispatch(ResetClaims());
    store.dispatch(AppendHandScore(scores));
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
    store.dispatch(AppendTrick(trick));
  });
  ctx.trickTracker.on("reset", () => {
    store.dispatch(ResetTricks());
  });
  ctx.passTracker.on("pass", pass => {
    store.dispatch(SetLocalPass(pass));
  });
  ReactDOM.render(
    <Provider store={store}>
      <GameApp userDispatcher={userDispatcher} />
    </Provider>,
    document.getElementById("app-container")!
  );
});
