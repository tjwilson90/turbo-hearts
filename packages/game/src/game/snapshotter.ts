import { EventData, Seat, Pass } from "../types";
import {
  TurboHearts,
  emptyStateSnapshot,
  newPlayer,
  withDeal,
  withToPlay,
  withSentPass,
  withReceivePass,
  withCharge,
  withPlay,
  withEndTrick
} from "./stateSnapshot";

export const SEATS: Seat[] = ["north", "east", "south", "west"];

const PASS_POSITION_OFFSETS: { [pass in Pass]: number } = {
  left: 1,
  right: -1,
  across: 2,
  keeper: 0
};

function addToSeat(seat: Seat, n: number): Seat {
  let i = SEATS.indexOf(seat) + n;
  if (i < 0) {
    i += SEATS.length;
  }
  i = i % SEATS.length;
  return SEATS[i];
}

export class Snapshotter {
  private snapshots: TurboHearts.StateSnapshot[] = [];

  constructor(private userName: string) {
    this.snapshots.push(emptyStateSnapshot(userName));
  }

  public onEvent = (event: EventData) => {
    const previous = this.snapshots[this.snapshots.length - 1];
    switch (event.type) {
      case "sit": {
        this.snapshots.push({
          ...previous,
          event,
          north: newPlayer(event.north.type, event.north.name),
          east: newPlayer(event.east.type, event.east.name),
          south: newPlayer(event.south.type, event.south.name),
          west: newPlayer(event.west.type, event.west.name)
        });
        break;
      }

      case "deal": {
        this.snapshots.push({
          ...previous,
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
          event,
          north: withToPlay(previous.north, !event.northDone),
          east: withToPlay(previous.east, !event.eastDone),
          south: withToPlay(previous.south, !event.southDone),
          west: withToPlay(previous.west, !event.westDone)
        });
        break;
      }

      case "send_pass": {
        const fromPlayer = previous[event.from];
        this.snapshots.push({
          ...previous,
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
          event,
          [event.to]: passPlayers.to,
          [fromSeat]: passPlayers.from
        });
        break;
      }

      case "charge_status": {
        this.snapshots.push({
          ...previous,
          event,
          north: withToPlay(previous.north, !event.northDone),
          east: withToPlay(previous.east, !event.eastDone),
          south: withToPlay(previous.south, !event.southDone),
          west: withToPlay(previous.west, !event.westDone)
        });
        break;
      }

      case "charge": {
        const chargePlayer = previous[event.seat];
        this.snapshots.push({
          ...previous,
          event,
          [event.seat]: withCharge(chargePlayer, event.cards)
        });
        break;
      }

      case "play_status": {
        const player = previous[event.nextPlayer];
        this.snapshots.push({
          ...previous,
          event,
          north: withToPlay(previous.north, false),
          east: withToPlay(previous.east, false),
          south: withToPlay(previous.south, false),
          west: withToPlay(previous.west, false),
          [event.nextPlayer]: withToPlay(player, true, event.legalPlays)
        });
        break;
      }

      case "play": {
        const player = previous[event.seat];
        this.snapshots.push({
          ...previous,
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
          event,
          north: withEndTrick(previous.north, allPlays, previous.north === winner),
          east: withEndTrick(previous.east, allPlays, previous.east === winner),
          south: withEndTrick(previous.south, allPlays, previous.south === winner),
          west: withEndTrick(previous.west, allPlays, previous.west === winner)
        });
      }
    }
    const next = this.snapshots[this.snapshots.length - 1];
    if (next !== previous) {
      console.log(next);
    }
  };
}
