import { TurboHearts, Player } from "../game/TurboHearts";
import { SpriteCard, Seat } from "../types";

interface PlayerAccessor {
  (th: TurboHearts): Player;
}

const TOP_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.topPlayer;
const RIGHT_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) =>
  th.rightPlayer;
const BOTTOM_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) =>
  th.bottomPlayer;
const LEFT_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.leftPlayer;

const playerAccessors: {
  [bottomSeat: string]: { [passFrom: string]: PlayerAccessor };
} = {};
playerAccessors["north"] = {};
playerAccessors["north"]["north"] = BOTTOM_PLAYER_ACCESSOR;
playerAccessors["north"]["east"] = LEFT_PLAYER_ACCESSOR;
playerAccessors["north"]["south"] = TOP_PLAYER_ACCESSOR;
playerAccessors["north"]["west"] = RIGHT_PLAYER_ACCESSOR;
playerAccessors["east"] = {};
playerAccessors["east"]["north"] = RIGHT_PLAYER_ACCESSOR;
playerAccessors["east"]["east"] = BOTTOM_PLAYER_ACCESSOR;
playerAccessors["east"]["south"] = LEFT_PLAYER_ACCESSOR;
playerAccessors["east"]["west"] = TOP_PLAYER_ACCESSOR;
playerAccessors["south"] = {};
playerAccessors["south"]["north"] = TOP_PLAYER_ACCESSOR;
playerAccessors["south"]["east"] = RIGHT_PLAYER_ACCESSOR;
playerAccessors["south"]["south"] = BOTTOM_PLAYER_ACCESSOR;
playerAccessors["south"]["west"] = LEFT_PLAYER_ACCESSOR;
playerAccessors["west"] = {};
playerAccessors["west"]["north"] = LEFT_PLAYER_ACCESSOR;
playerAccessors["west"]["east"] = TOP_PLAYER_ACCESSOR;
playerAccessors["west"]["south"] = RIGHT_PLAYER_ACCESSOR;
playerAccessors["west"]["west"] = BOTTOM_PLAYER_ACCESSOR;

export function getPlayerAccessor(bottomSeat: Seat, seat: Seat) {
  return playerAccessors[bottomSeat][seat];
}
