import { Seat, Position, Pass } from "../types";

/**
 * The standard order of positions, clockwise from Top.
 */
export const POSITION_ORDER: Position[] = ["top", "right", "bottom", "left"];

/**
 * The standard order of seats, clockwise from North.
 */
export const SEAT_ORDER: Seat[] = ["north", "east", "south", "west"];

/**
 * Position or Seat offset for a given pass
 * @see #addToSeat()
 */
export const PASS_OFFSETS: { [pass in Pass]: number } = {
  left: 1,
  right: -1,
  across: 2,
  keeper: 0
};

/**
 * Given an offset from a Seat, find the new Seat.
 * @param seat source Seat
 * @param n -4 <= n < 4
 */
export function addToSeat(seat: Seat, n: number): Seat {
  let i = SEAT_ORDER.indexOf(seat) + n;
  if (i < 0) {
    i += SEAT_ORDER.length;
  }
  i = i % SEAT_ORDER.length;
  return SEAT_ORDER[i];
}

export function subtractSeats(left: Seat, right: Seat): number {
  let n = SEAT_ORDER.indexOf(left) - SEAT_ORDER.indexOf(right);
  return n < 0 ? n + SEAT_ORDER.length : n;
}

/**
 * A map from a bottom Seat to all Seats in Top, Right, Bottom, Left order.
 */
export const SEAT_ORDER_FOR_BOTTOM_SEAT: { [bottomSeat in Seat]: Seat[] } = {
  // [top, right, bottom, left]
  north: ["south", "west", "north", "east"],
  east: ["west", "north", "east", "south"],
  south: ["north", "east", "south", "west"],
  west: ["east", "south", "west", "north"]
};

/**
 * A map from a bottom Seat to all Positions in NESW order.
 */
export const POSITION_FOR_BOTTOM_SEAT: { [bottomSeat in Seat]: Position[] } = {
  north: ["bottom", "left", "top", "right"],
  east: ["right", "bottom", "left", "top"],
  south: ["top", "right", "bottom", "left"],
  west: ["left", "top", "right", "bottom"]
};

/**
 * A map from bottom Seat and true Seat to Position.
 */
export const POSITIONS: { [bottomSeat in Seat]: { [trueSeat in Seat]: Position } } = {
  north: {
    north: "bottom",
    east: "left",
    south: "top",
    west: "right"
  },
  east: {
    north: "right",
    east: "bottom",
    south: "left",
    west: "top"
  },
  south: {
    north: "top",
    east: "right",
    south: "bottom",
    west: "left"
  },
  west: {
    north: "left",
    east: "top",
    south: "right",
    west: "bottom"
  }
};
