import { Dispatch } from "redoodle";
import { SitEventData, Seat } from "../types";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { SetGameUsers, UpdateUsers } from "./actions";

export function getBottomSeat(event: SitEventData, myUserId: string): Seat {
  if (event.north.userId === myUserId) {
    return "north";
  } else if (event.east.userId === myUserId) {
    return "east";
  } else if (event.south.userId === myUserId) {
    return "south";
  } else if (event.west.userId === myUserId) {
    return "west";
  } else {
    return "south";
  }
}

const BOTTOM_SEAT_TO_POSITION_INDICES = {
  north: [2, 3, 0, 1],
  east: [3, 0, 1, 2],
  south: [0, 1, 2, 3],
  west: [1, 2, 3, 0]
};

function bot(id: string) {
  return {
    userId: id,
    name: `Bot (${id.substring(0, 8)})`
  };
}

export class UserDispatcher {
  private loadedIds = new Set<string>();

  constructor(private service: TurboHeartsService, private myUserId: string, private dispatch: Dispatch) {}

  public async loadUsersForGame(event: SitEventData) {
    const ids = [event.north.userId, event.east.userId, event.south.userId, event.west.userId];
    for (const id of ids) {
      this.loadedIds.add(id);
    }
    const loadedUsers = await this.service.getUsers(ids);
    const bottomSeat = getBottomSeat(event, this.myUserId);
    const topId = ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][0]];
    const rightId = ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][1]];
    const bottomId = ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][2]];
    const leftId = ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][3]];
    const usersByPosition = {
      north: loadedUsers[ids[0]] ?? bot(ids[0]),
      east: loadedUsers[ids[1]] ?? bot(ids[1]),
      south: loadedUsers[ids[2]] ?? bot(ids[2]),
      west: loadedUsers[ids[3]] ?? bot(ids[3]),
      bottomSeat,
      top: loadedUsers[topId] ?? bot(topId),
      right: loadedUsers[rightId] ?? bot(rightId),
      bottom: loadedUsers[bottomId] ?? bot(bottomId),
      left: loadedUsers[leftId] ?? bot(leftId)
    };
    this.dispatch(SetGameUsers(usersByPosition));
    this.dispatch(UpdateUsers(loadedUsers));
  }

  public async loadUsers(ids: string[]) {
    const toLoad = ids.filter(id => !this.loadedIds.has(id));
    if (toLoad.length === 0) {
      return;
    }
    for (const id of toLoad) {
      this.loadedIds.add(id);
    }
    const loadedUsers = await this.service.getUsers(toLoad);
    this.dispatch(UpdateUsers(loadedUsers));
  }
}
