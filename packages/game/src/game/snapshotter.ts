import { EventEmitter } from "eventemitter3";
import { EventData } from "../types";
import { PASS_OFFSETS, addToSeat } from "../util/seatPositions";
import {
  emptyStateSnapshot,
  newPlayer,
  TurboHearts,
  withAction,
  withCharge,
  withDeal,
  withEndTrick,
  withPlay,
  withReceivePass,
  withSentPass,
  withHiddenReceivePass,
  withHiddenSentPass
} from "./stateSnapshot";

export class Snapshotter {
  private emitter = new EventEmitter();
  private snapshots: TurboHearts.StateSnapshot[] = [];

  constructor(userId: string) {
    this.snapshots.push(emptyStateSnapshot(userId));
  }

  public onEvent = (event: EventData) => {
    const previous = this.snapshots[this.snapshots.length - 1];
    switch (event.type) {
      case "sit": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          north: newPlayer(event.north.type, event.north.userId),
          east: newPlayer(event.east.type, event.east.userId),
          south: newPlayer(event.south.type, event.south.userId),
          west: newPlayer(event.west.type, event.west.userId)
        });
        break;
      }

      case "deal": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          pass: event.pass,
          north: withDeal(previous.north, event.north),
          east: withDeal(previous.east, event.east),
          south: withDeal(previous.south, event.south),
          west: withDeal(previous.west, event.west)
        });
        break;
      }

      case "pass_status": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          north: withAction(previous.north, event.northDone ? "none" : "pass"),
          east: withAction(previous.east, event.eastDone ? "none" : "pass"),
          south: withAction(previous.south, event.southDone ? "none" : "pass"),
          west: withAction(previous.west, event.westDone ? "none" : "pass")
        });
        break;
      }

      case "send_pass": {
        const fromPlayer = previous[event.from];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.from]: withSentPass(fromPlayer, event.cards)
        });
        break;
      }

      case "recv_pass": {
        const toPlayer = previous[event.to];
        const fromSeat = addToSeat(event.to, -PASS_OFFSETS[previous.pass]);
        const fromPlayer = previous[fromSeat];
        const passPlayers = withReceivePass(fromPlayer, toPlayer, event.cards);
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.to]: passPlayers.to,
          [fromSeat]: passPlayers.from
        });
        break;
      }

      case "hidden_send_pass": {
        const fromPlayer = previous[event.from];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.from]: withHiddenSentPass(fromPlayer, event.count)
        });
        break;
      }

      case "hidden_recv_pass": {
        const toPlayer = previous[event.to];
        const fromSeat = addToSeat(event.to, -PASS_OFFSETS[previous.pass]);
        const fromPlayer = previous[fromSeat];
        const passPlayers = withHiddenReceivePass(fromPlayer, toPlayer, event.count);
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.to]: passPlayers.to,
          [fromSeat]: passPlayers.from
        });
        break;
      }

      case "charge_status": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          north: withAction(previous.north, event.northDone ? "none" : "charge"),
          east: withAction(previous.east, event.eastDone ? "none" : "charge"),
          south: withAction(previous.south, event.southDone ? "none" : "charge"),
          west: withAction(previous.west, event.westDone ? "none" : "charge")
        });
        break;
      }

      case "charge": {
        const chargePlayer = previous[event.seat];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.seat]: withCharge(chargePlayer, event.cards)
        });
        break;
      }

      case "play_status": {
        const player = previous[event.nextPlayer];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          north: withAction(previous.north, "none"),
          east: withAction(previous.east, "none"),
          south: withAction(previous.south, "none"),
          west: withAction(previous.west, "none"),
          [event.nextPlayer]: withAction(player, "play", event.legalPlays)
        });
        break;
      }

      case "play": {
        const player = previous[event.seat];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          playNumber: previous.playNumber + 1,
          event,
          [event.seat]: withPlay(player, event.card)
        });
        break;
      }

      case "claim": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          [event.seat]: withDeal(previous[event.seat], event.hand)
        });
        break;
      }

      case "end_trick": {
        const winner = previous[event.winner];
        const allPlays = [
          ...previous.north.plays,
          ...previous.east.plays,
          ...previous.south.plays,
          ...previous.west.plays
        ];
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          trickNumber: previous.trickNumber + 1,
          playNumber: 0,
          event,
          north: withEndTrick(previous.north, allPlays, previous.north === winner),
          east: withEndTrick(previous.east, allPlays, previous.east === winner),
          south: withEndTrick(previous.south, allPlays, previous.south === winner),
          west: withEndTrick(previous.west, allPlays, previous.west === winner)
        });
        break;
      }

      case "game_complete": {
        this.snapshots.push({
          ...previous,
          index: previous.index + 1,
          event,
          north: withAction(withDeal(previous.north, []), "none"),
          east: withAction(withDeal(previous.east, []), "none"),
          south: withAction(withDeal(previous.south, []), "none"),
          west: withAction(withDeal(previous.west, []), "none")
        });
        break;
      }
    }
    const next = this.snapshots[this.snapshots.length - 1];
    if (next !== previous) {
      this.emitter.emit("snapshot", { next, previous });
    }
  };

  public on(
    _event: "snapshot",
    fn: (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => void
  ) {
    this.emitter.on("snapshot", fn);
  }

  public off(
    _event: "snapshot",
    fn: (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => void
  ) {
    this.emitter.off("snapshot", fn);
  }
}
