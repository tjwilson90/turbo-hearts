export type Pass = "Left" | "Right" | "Across" | "Keep";

export interface DealEventData {
  type: "deal";
  north: Card[];
  east: Card[];
  south: Card[];
  west: Card[];
  pass: Pass;
}

export type Seat = "north" | "east" | "south" | "west";

export interface SendPassData {
  type: "send_pass";
  from: Seat;
  cards: Card[];
}

export interface Event {
  begin(): void;
  isFinished(): boolean;
}

export interface Point {
  x: number;
  y: number;
}

export interface SpriteCard {
  card: Card;
  sprite: PIXI.Sprite;
  hidden: boolean;
}

export type EventData = DealEventData | SendPassData;

export type Card =
  | "BACK"
  | "2C"
  | "3C"
  | "4C"
  | "5C"
  | "6C"
  | "7C"
  | "8C"
  | "9C"
  | "TC"
  | "JC"
  | "QC"
  | "KC"
  | "AC"
  | "2D"
  | "3D"
  | "4D"
  | "5D"
  | "6D"
  | "7D"
  | "8D"
  | "9D"
  | "TD"
  | "JD"
  | "QD"
  | "KD"
  | "AD"
  | "2H"
  | "3H"
  | "4H"
  | "5H"
  | "6H"
  | "7H"
  | "8H"
  | "9H"
  | "TH"
  | "JH"
  | "QH"
  | "KH"
  | "AH"
  | "2S"
  | "3S"
  | "4S"
  | "5S"
  | "6S"
  | "7S"
  | "8S"
  | "9S"
  | "TS"
  | "JS"
  | "QS"
  | "KS"
  | "AS";

export const CARDS = [
  "BACK",
  "2C",
  "3C",
  "4C",
  "5C",
  "6C",
  "7C",
  "8C",
  "9C",
  "TC",
  "JC",
  "QC",
  "KC",
  "AC",
  "2D",
  "3D",
  "4D",
  "5D",
  "6D",
  "7D",
  "8D",
  "9D",
  "TD",
  "JD",
  "QD",
  "KD",
  "AD",
  "2H",
  "3H",
  "4H",
  "5H",
  "6H",
  "7H",
  "8H",
  "9H",
  "TH",
  "JH",
  "QH",
  "KH",
  "AH",
  "2S",
  "3S",
  "4S",
  "5S",
  "6S",
  "7S",
  "8S",
  "9S",
  "TS",
  "JS",
  "QS",
  "KS",
  "AS"
];
