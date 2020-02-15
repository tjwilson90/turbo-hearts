export const CARD_SCALE = 0.5;
export const CARD_NATIVE_WIDTH = 212;
export const CARD_NATIVE_HEIGHT = 329;
export const CARD_DISPLAY_WIDTH = CARD_NATIVE_WIDTH * CARD_SCALE;
export const CARD_DISPLAY_HEIGHT = CARD_NATIVE_HEIGHT * CARD_SCALE;
export const CARD_OVERLAP = 52 * CARD_SCALE;

export const TABLE_SIZE = 1000;
export const TABLE_CENTER_X = TABLE_SIZE / 2;
export const TABLE_CENTER_Y = TABLE_SIZE / 2;

export const ANIMATION_DURATION = 400;
export const ANIMATION_DELAY = 50;

export const TABLE_CARD_UNDERLAP = 0.2;

export const TOP_X = TABLE_CENTER_X;
export const TOP_Y = CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const TOP_ROTATION = Math.PI;
export const TOP = { x: TOP_X, y: TOP_Y, rotation: TOP_ROTATION };

export const RIGHT_X = TABLE_SIZE - CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const RIGHT_Y = TABLE_CENTER_Y;
export const RIGHT_ROTATION = -Math.PI / 2;
export const RIGHT = { x: RIGHT_X, y: RIGHT_Y, rotation: RIGHT_ROTATION };

export const BOTTOM_X = TABLE_CENTER_X;
export const BOTTOM_Y = TABLE_SIZE - CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const BOTTOM_ROTATION = 0;
export const BOTTOM = { x: BOTTOM_X, y: BOTTOM_Y, rotation: BOTTOM_ROTATION };

export const LEFT_X = CARD_DISPLAY_HEIGHT * TABLE_CARD_UNDERLAP;
export const LEFT_Y = TABLE_CENTER_Y;
export const LEFT_ROTATION = Math.PI / 2;
export const LEFT = { x: LEFT_X, y: LEFT_Y, rotation: LEFT_ROTATION };

const CORNER_OFFSET = CARD_DISPLAY_HEIGHT * (4 * TABLE_CARD_UNDERLAP);
export const TOP_RIGHT_X = TABLE_SIZE - CORNER_OFFSET;
export const TOP_RIGHT_Y = CORNER_OFFSET;
export const TOP_RIGHT_ROTATION = TOP_ROTATION + Math.PI / 4;
export const TOP_RIGHT = {
  x: TOP_RIGHT_X,
  y: TOP_RIGHT_Y,
  rotation: TOP_RIGHT_ROTATION
};

export const BOTTOM_RIGHT_X = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_RIGHT_Y = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_RIGHT_ROTATION = RIGHT_ROTATION + Math.PI / 4;
export const BOTTOM_RIGHT = {
  x: BOTTOM_RIGHT_X,
  y: BOTTOM_RIGHT_Y,
  rotation: BOTTOM_RIGHT_ROTATION
};

export const BOTTOM_LEFT_X = CORNER_OFFSET;
export const BOTTOM_LEFT_Y = TABLE_SIZE - CORNER_OFFSET;
export const BOTTOM_LEFT_ROTATION = BOTTOM_ROTATION + Math.PI / 4;
export const BOTTOM_LEFT = {
  x: BOTTOM_LEFT_X,
  y: BOTTOM_LEFT_Y,
  rotation: BOTTOM_LEFT_ROTATION
};

export const TOP_LEFT_X = CORNER_OFFSET;
export const TOP_LEFT_Y = CORNER_OFFSET;
export const TOP_LEFT_ROTATION = LEFT_ROTATION + Math.PI / 4;
export const TOP_LEFT = {
  x: TOP_LEFT_X,
  y: TOP_LEFT_Y,
  rotation: TOP_LEFT_ROTATION
};