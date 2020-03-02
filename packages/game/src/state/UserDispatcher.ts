import { Dispatch } from "redoodle";
import { SitEventData } from "../types";
import { TurboHeartsService } from "../game/TurboHeartsService";
import { SetUsers } from "./actions";

function getBottomSeat(event: SitEventData) {
  if (event.north.userId === this.userId) {
    return "north";
  } else if (event.east.userId === this.userId) {
    return "east";
  } else if (event.south.userId === this.userId) {
    return "south";
  } else if (event.west.userId === this.userId) {
    return "west";
  } else {
    return "south";
  }
}

const BOTTOM_SEAT_TO_POSITION_INDICES = {
  north: [2, 3, 0, 1],
  east: [1, 2, 3, 0],
  south: [0, 1, 2, 3],
  west: [3, 0, 1, 2]
};

export class UserDispatcher {
  constructor(private service: TurboHeartsService, private dispatch: Dispatch) {}

  public async loadUsersForGame(event: SitEventData) {
    const ids = [event.north.userId, event.east.userId, event.south.userId, event.west.userId];
    const loadedUsers = await this.service.getUsers(ids);
    const bottomSeat = getBottomSeat(event);
    const usersByPosition = {
      top: loadedUsers[ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][0]]],
      right: loadedUsers[ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][1]]],
      bottom: loadedUsers[ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][2]]],
      left: loadedUsers[ids[BOTTOM_SEAT_TO_POSITION_INDICES[bottomSeat][3]]]
    };
    this.dispatch(SetUsers(usersByPosition));
  }
}
