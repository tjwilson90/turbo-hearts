import { DropShadowFilter } from "@pixi/filter-drop-shadow";
import { PlayerCardPositions } from "./types";

export const Z_BACKGROUND = 0;
export const Z_PILE_CARDS = 100;
export const Z_LIMBO_CARDS = 200;
export const Z_DEALING_CARDS = 300;
export const Z_PLAYED_CARDS = 400;
export const Z_CHARGED_CARDS = 500;
export const Z_HAND_CARDS = 600;
export const Z_TRANSIT_CARDS = 700;
export const Z_NAMEPLATE = 800;
export const Z_BUTTON = 900;

export const CARD_SCALE = 0.5;
export const CARD_NATIVE_WIDTH = 212;
export const CARD_NATIVE_HEIGHT = 329;
export const CARD_DISPLAY_WIDTH = CARD_NATIVE_WIDTH * CARD_SCALE;
export const CARD_DISPLAY_HEIGHT = CARD_NATIVE_HEIGHT * CARD_SCALE;
export const CARD_OVERLAP = 52 * CARD_SCALE;

export const TABLE_SIZE = 768;
export const TABLE_CENTER_X = TABLE_SIZE / 2;
export const TABLE_CENTER_Y = TABLE_SIZE / 2;

export const ANIMATION_DURATION = 600;
export const ANIMATION_DELAY = 50;

export const FAST_ANIMATION_DURATION = ANIMATION_DURATION / 2;
export const FAST_ANIMATION_DELAY = ANIMATION_DELAY / 2;

export const FASTER_ANIMATION_DURATION = ANIMATION_DURATION / 4;

export const TRICK_COLLECTION_PAUSE = 1000;
export const CLAIM_PAUSE = 75;

export const TABLE_CARD_UNDERLAP = -0.1;
export const CARD_MARGIN = 10;
export const CHARGE_OFFSET = CARD_DISPLAY_HEIGHT * (1 + TABLE_CARD_UNDERLAP) + CARD_MARGIN;
export const CHARGE_OVERLAP = CARD_DISPLAY_WIDTH + CARD_MARGIN;

export const TOP_X = TABLE_CENTER_X;
export const TOP_Y = CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const TOP_ROTATION = Math.PI;
export const TOP_CHARGE_X = TOP_X;
export const TOP_CHARGE_Y = CHARGE_OFFSET;
export const TOP_PLAY_X = TOP_X;
export const TOP_PLAY_Y = CHARGE_OFFSET + CARD_DISPLAY_HEIGHT + CARD_MARGIN;
export const TOP: PlayerCardPositions = {
  x: TOP_X,
  y: TOP_Y,
  chargeX: TOP_CHARGE_X,
  chargeY: TOP_CHARGE_Y,
  playX: TOP_PLAY_X,
  playY: TOP_PLAY_Y,
  pileX: TOP_X,
  pileY: TOP_Y + CARD_DISPLAY_HEIGHT / 2,
  pileRotation: Math.PI / 2,
  rotation: TOP_ROTATION
};

export const RIGHT_X = TABLE_SIZE - CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const RIGHT_Y = TABLE_CENTER_Y;
export const RIGHT_ROTATION = -Math.PI / 2;
export const RIGHT_CHARGE_X = TABLE_SIZE - CHARGE_OFFSET;
export const RIGHT_CHARGE_Y = TABLE_CENTER_Y;
export const RIGHT_PLAY_X = TABLE_SIZE - (CHARGE_OFFSET + CARD_DISPLAY_HEIGHT + CARD_MARGIN);
export const RIGHT_PLAY_Y = TABLE_CENTER_Y;
export const RIGHT: PlayerCardPositions = {
  x: RIGHT_X,
  y: RIGHT_Y,
  chargeX: RIGHT_CHARGE_X,
  chargeY: RIGHT_CHARGE_Y,
  playX: RIGHT_PLAY_X,
  playY: RIGHT_PLAY_Y,
  pileX: RIGHT_X - CARD_DISPLAY_HEIGHT / 2,
  pileY: RIGHT_Y,
  pileRotation: 0,
  rotation: RIGHT_ROTATION
};

export const BOTTOM_X = TABLE_CENTER_X;
export const BOTTOM_Y = TABLE_SIZE - CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const BOTTOM_ROTATION = 0;
export const BOTTOM_CHARGE_X = TABLE_CENTER_X;
export const BOTTOM_CHARGE_Y = TABLE_SIZE - CHARGE_OFFSET;
export const BOTTOM_PLAY_X = TABLE_CENTER_X;
export const BOTTOM_PLAY_Y = TABLE_SIZE - (CHARGE_OFFSET + CARD_DISPLAY_HEIGHT + CARD_MARGIN);
export const BOTTOM: PlayerCardPositions = {
  x: BOTTOM_X,
  y: BOTTOM_Y,
  chargeX: BOTTOM_CHARGE_X,
  chargeY: BOTTOM_CHARGE_Y,
  playX: BOTTOM_PLAY_X,
  playY: BOTTOM_PLAY_Y,
  pileX: BOTTOM_X,
  pileY: BOTTOM_Y - CARD_DISPLAY_HEIGHT / 2,
  pileRotation: Math.PI / 2,
  rotation: BOTTOM_ROTATION
};

export const LEFT_X = CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const LEFT_Y = TABLE_CENTER_Y;
export const LEFT_ROTATION = Math.PI / 2;
export const LEFT_CHARGE_X = CHARGE_OFFSET;
export const LEFT_CHARGE_Y = TABLE_CENTER_Y;
export const LEFT_PLAY_X = CHARGE_OFFSET + CARD_DISPLAY_HEIGHT + CARD_MARGIN;
export const LEFT_PLAY_Y = TABLE_CENTER_Y;
export const LEFT: PlayerCardPositions = {
  x: LEFT_X,
  y: LEFT_Y,
  chargeX: LEFT_CHARGE_X,
  chargeY: LEFT_CHARGE_Y,
  playX: LEFT_PLAY_X,
  playY: LEFT_PLAY_Y,
  pileX: LEFT_X + CARD_DISPLAY_HEIGHT / 2,
  pileY: LEFT_Y,
  pileRotation: 0,
  rotation: LEFT_ROTATION
};

const CORNER_OFFSET = CARD_DISPLAY_HEIGHT;
export const TOP_RIGHT_X = TABLE_SIZE - CORNER_OFFSET;
export const TOP_RIGHT_Y = CORNER_OFFSET;
export const TOP_RIGHT_ROTATION = TOP_ROTATION + Math.PI / 4;
export const LIMBO_TOP_RIGHT = {
  x: TOP_RIGHT_X,
  y: TOP_RIGHT_Y,
  rotation: TOP_RIGHT_ROTATION
};

export const BOTTOM_RIGHT_X = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_RIGHT_Y = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_RIGHT_ROTATION = RIGHT_ROTATION + Math.PI / 4;
export const LIMBO_BOTTOM_RIGHT = {
  x: BOTTOM_RIGHT_X,
  y: BOTTOM_RIGHT_Y,
  rotation: BOTTOM_RIGHT_ROTATION
};

export const BOTTOM_LEFT_X = CORNER_OFFSET;
export const BOTTOM_LEFT_Y = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_LEFT_ROTATION = BOTTOM_ROTATION + Math.PI / 4;
export const LIMBO_BOTTOM_LEFT = {
  x: BOTTOM_LEFT_X,
  y: BOTTOM_LEFT_Y,
  rotation: BOTTOM_LEFT_ROTATION
};

export const TOP_LEFT_X = CORNER_OFFSET;
export const TOP_LEFT_Y = CORNER_OFFSET;
export const TOP_LEFT_ROTATION = LEFT_ROTATION + Math.PI / 4;
export const LIMBO_TOP_LEFT = {
  x: TOP_LEFT_X,
  y: TOP_LEFT_Y,
  rotation: TOP_LEFT_ROTATION
};

export const LIMBO_TOP = {
  x: TOP_X,
  y: TOP_Y + CARD_DISPLAY_HEIGHT + CARD_MARGIN,
  rotation: TOP_ROTATION
};

export const LIMBO_RIGHT = {
  x: RIGHT_X - (CARD_DISPLAY_HEIGHT + CARD_MARGIN),
  y: RIGHT_Y,
  rotation: RIGHT_ROTATION
};

export const LIMBO_BOTTOM = {
  x: BOTTOM_X,
  y: BOTTOM_Y - (CARD_DISPLAY_HEIGHT + CARD_MARGIN),
  rotation: BOTTOM_ROTATION
};

export const LIMBO_LEFT = {
  x: LEFT_X + CARD_DISPLAY_HEIGHT + CARD_MARGIN,
  y: LEFT_Y,
  rotation: LEFT_ROTATION
};

export const LIMBO_CENTER = {
  x: TABLE_CENTER_X,
  y: TABLE_CENTER_Y,
  rotation: 0
};

export const CARD_DROP_SHADOW = new DropShadowFilter({
  distance: 0.5,
  alpha: 0.4,
  pixelSize: 1,
  blur: 1,
  resolution: window.devicePixelRatio
});
