import { EventData, Pass, Seat } from "../types";
import {
  emptyStateSnapshot,
  newPlayer,
  TurboHearts,
  withCharge,
  withDeal,
  withEndTrick,
  withPlay,
  withReceivePass,
  withSentPass,
  withAction
} from "./stateSnapshot";
import { EventEmitter } from "eventemitter3";

export const SEATS: Seat[] = ["north", "east", "south", "west"];

export const PASS_POSITION_OFFSETS: { [pass in Pass]: number } = {
  left: 1,
  right: -1,
  across: 2,
  keeper: 0
};

export function addToSeat(seat: Seat, n: number): Seat {
  let i = SEATS.indexOf(seat) + n;
  if (i < 0) {
    i += SEATS.length;
  }
  i = i % SEATS.length;
  return SEATS[i];
}

export class Snapshotter {
  private emitter = new EventEmitter();
  private snapshots: TurboHearts.StateSnapshot[] = [];

  constructor(userName: string) {
    this.snapshots.push(emptyStateSnapshot(userName));
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
        const fromSeat = addToSeat(event.to, -PASS_POSITION_OFFSETS[previous.pass]);
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
    }
    const next = this.snapshots[this.snapshots.length - 1];
    if (next !== previous) {
      this.emitter.emit("snapshot", { next, previous });
    }
  };

  public on(
    event: "snapshot",
    fn: (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => void
  ) {
    this.emitter.on("snapshot", fn);
  }

  public off(
    event: "snapshot",
    fn: (event: { next: TurboHearts.StateSnapshot; previous: TurboHearts.StateSnapshot }) => void
  ) {
    this.emitter.off("snapshot", fn);
  }
}
