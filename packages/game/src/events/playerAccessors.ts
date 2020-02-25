import { TurboHearts, Player } from "../game/TurboHearts";
import { SpriteCard, Seat } from "../types";

interface PlayerAccessor {
  (th: TurboHearts): Player;
}

const TOP_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.topPlayer;
const RIGHT_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.rightPlayer;
const BOTTOM_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.bottomPlayer;
const LEFT_PLAYER_ACCESSOR: PlayerAccessor = (th: TurboHearts) => th.leftPlayer;

const playerAccessors: {
  [bottomSeat in Seat]: { [trueSeat in Seat]: PlayerAccessor };
} = {
  north: {
    north: BOTTOM_PLAYER_ACCESSOR,
    east: LEFT_PLAYER_ACCESSOR,
    south: TOP_PLAYER_ACCESSOR,
    west: RIGHT_PLAYER_ACCESSOR
  },
  east: {
    north: RIGHT_PLAYER_ACCESSOR,
    east: BOTTOM_PLAYER_ACCESSOR,
    south: LEFT_PLAYER_ACCESSOR,
    west: TOP_PLAYER_ACCESSOR
  },
  south: {
    north: TOP_PLAYER_ACCESSOR,
    east: RIGHT_PLAYER_ACCESSOR,
    south: BOTTOM_PLAYER_ACCESSOR,
    west: LEFT_PLAYER_ACCESSOR
  },
  west: {
    north: LEFT_PLAYER_ACCESSOR,
    east: TOP_PLAYER_ACCESSOR,
    south: RIGHT_PLAYER_ACCESSOR,
    west: BOTTOM_PLAYER_ACCESSOR
  }
};

export function getPlayerAccessor(bottomSeat: Seat, trueSeat: Seat) {
  return playerAccessors[bottomSeat][trueSeat];
}
