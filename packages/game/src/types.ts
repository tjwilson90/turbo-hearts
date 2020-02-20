export type Rules = "classic";
export type Pass = "left" | "right" | "across" | "keeper";
export type Seat = "north" | "east" | "south" | "west";
export type Position = "top" | "right" | "bottom" | "left";

interface SitPlayer {
  name: string;
  type: "bot" | "human";
  algorithm?: string;
}

export interface SitEventData {
  type: "sit";
  north: SitPlayer;
  east: SitPlayer;
  south: SitPlayer;
  west: SitPlayer;
  rules: Rules;
}

export interface DealEventData {
  type: "deal";
  north: Card[];
  east: Card[];
  south: Card[];
  west: Card[];
  pass: Pass;
}

export interface StartPassingEventData {
  type: "start_passing";
}

export interface SendPassEventData {
  type: "send_pass";
  from: Seat;
  cards: Card[];
}

export interface ReceivePassEventData {
  type: "recv_pass";
  to: Seat;
  cards: Card[];
}

export interface StartChargingEventData {
  type: "start_charging";
}

export interface ChargeEventData {
  type: "charge";
  seat: Seat;
  cards: Card[];
}

export interface StartTrickEventData {
  type: "start_trick";
  leader: Seat;
}

export interface YourPlayEventData {
  type: "your_play";
  legalPlays: Card[];
}

export interface PlayEventData {
  type: "play";
  seat: Seat;
  card: Card;
}

export interface EndTrickEventData {
  type: "end_trick";
  winner: Seat;
}

export interface GameCompleteEventData {
  type: "game_complete";
}

export interface Event {
  type: EventData["type"];
  begin(): void;
  isFinished(): boolean;

  // updateState(): void;
  // animate(): Promise<void>;
}

export interface Point {
  x: number;
  y: number;
}

export interface PointWithRotation extends Point {
  rotation: number;
}

export interface PlayerCardPositions extends PointWithRotation {
  chargeX: number;
  chargeY: number;
  playX: number;
  playY: number;
  pileX: number;
  pileY: number;
  pileRotation: number;
}

export interface SpriteCard {
  card: Card;
  sprite: PIXI.Sprite;
  hidden: boolean;
}

export type EventData =
  | SitEventData
  | DealEventData
  | StartPassingEventData
  | SendPassEventData
  | ReceivePassEventData
  | StartChargingEventData
  | ChargeEventData
  | StartTrickEventData
  | YourPlayEventData
  | PlayEventData
  | EndTrickEventData
  | GameCompleteEventData;

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