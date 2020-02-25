import { BOTTOM, LEFT, RIGHT, TOP } from "../const";
import { PlayerCardPositions, Seat } from "../types";

const handPositions: {
  [bottomSeat in Seat]: { [seat in Seat]: PlayerCardPositions };
} = {
  north: {
    north: BOTTOM,
    east: LEFT,
    south: TOP,
    west: RIGHT
  },
  east: {
    north: RIGHT,
    east: BOTTOM,
    south: LEFT,
    west: TOP
  },
  south: {
    north: TOP,
    east: RIGHT,
    south: BOTTOM,
    west: LEFT
  },
  west: {
    north: LEFT,
    east: TOP,
    south: RIGHT,
    west: BOTTOM
  }
};

export function getHandPosition(bottomSeat: Seat, seat: Seat) {
  return handPositions[bottomSeat][seat];
}
