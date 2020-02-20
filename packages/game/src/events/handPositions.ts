import { BOTTOM, LEFT, RIGHT, TOP } from "../const";
import { PlayerCardPositions, Seat } from "../types";

const handPositions: {
  [bottomSeat: string]: { [seat: string]: PlayerCardPositions };
} = {};
handPositions["north"] = {};
handPositions["north"]["north"] = BOTTOM;
handPositions["north"]["east"] = LEFT;
handPositions["north"]["south"] = TOP;
handPositions["north"]["west"] = RIGHT;
handPositions["east"] = {};
handPositions["east"]["north"] = RIGHT;
handPositions["east"]["east"] = BOTTOM;
handPositions["east"]["south"] = LEFT;
handPositions["east"]["west"] = TOP;
handPositions["south"] = {};
handPositions["south"]["north"] = TOP;
handPositions["south"]["east"] = RIGHT;
handPositions["south"]["south"] = BOTTOM;
handPositions["south"]["west"] = LEFT;
handPositions["west"] = {};
handPositions["west"]["north"] = LEFT;
handPositions["west"]["east"] = TOP;
handPositions["west"]["south"] = RIGHT;
handPositions["west"]["west"] = BOTTOM;

export function getHandPosition(bottomSeat: Seat, seat: Seat) {
  return handPositions[bottomSeat][seat];
}
