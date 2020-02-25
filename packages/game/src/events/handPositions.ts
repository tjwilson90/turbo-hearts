import { BOTTOM, LEFT, RIGHT, TOP } from "../const";
import { PlayerCardPositions, Seat, Position, EventData } from "../types";

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

export interface DirectionAccessor<T, R> {
  north: (input: T) => R;
  east: (input: T) => R;
  south: (input: T) => R;
  west: (input: T) => R;
}

const SEAT_MAP: {
  [bottomSeat in Seat]: { [position in Position]: Seat };
} = {
  south: {
    top: "south",
    right: "west",
    bottom: "north",
    left: "east"
  },
  west: {
    top: "west",
    right: "north",
    bottom: "east",
    left: "south"
  },
  north: {
    top: "north",
    right: "east",
    bottom: "south",
    left: "west"
  },
  east: {
    top: "east",
    right: "south",
    bottom: "west",
    left: "north"
  }
};

export function seatEventFunction<T, R>(
  bottomSeat: Seat,
  position: Position,
  accessor: DirectionAccessor<T, R>,
  input: T
) {
  return accessor[SEAT_MAP[bottomSeat][position]](input);
}
