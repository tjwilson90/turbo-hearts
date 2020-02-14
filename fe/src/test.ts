import { DealEventData } from "./types";

export const TEST_EVENTS = [
  //   {
  //     type: "sit",
  //     north: { type: "bot", name: "gdawson (bot)", algorithm: "random" },
  //     east: { type: "bot", name: "nhardin (bot)", algorithm: "random" },
  //     south: { type: "human", name: "dan" },
  //     west: { type: "bot", name: "mlove (bot)", algorithm: "random" },
  //     rules: "classic"
  //   },
  {
    type: "deal",
    north: [
      "KS",
      "QS",
      "QH",
      "TH",
      "7H",
      "6H",
      "3H",
      "2H",
      "9D",
      "8D",
      "QC",
      "9C",
      "6C"
    ],
    east: [
      "TS",
      "9S",
      "2S",
      "AH",
      "JH",
      "AD",
      "KD",
      "QD",
      "TD",
      "4D",
      "2D",
      "AC",
      "4C"
    ],
    south: [
      "JS",
      "8S",
      "7S",
      "6S",
      "5S",
      "3S",
      "9H",
      "5H",
      "7D",
      "5D",
      "3D",
      "KC",
      "3C"
    ],
    west: [
      "AS",
      "4S",
      "KH",
      "8H",
      "4H",
      "JD",
      "6D",
      "JC",
      "TC",
      "8C",
      "7C",
      "5C",
      "2C"
    ],
    pass: "Left"
  },
  //   { type: "start_passing" },
  { type: "send_pass", from: "north", cards: ["2H", "8D", "QC"] },
  { type: "send_pass", from: "west", cards: ["4S", "4H", "6D"] },
  //   { type: "recv_pass", to: "north", cards: ["4S", "4H", "6D"] },
  { type: "send_pass", from: "east", cards: ["2S", "AD", "AC"] },
  //   { type: "recv_pass", to: "east", cards: ["2H", "8D", "QC"] },
  { type: "send_pass", from: "south", cards: ["7D", "5D", "3D"] }
  //   { type: "recv_pass", to: "south", cards: ["2S", "AD", "AC"] },
  //   { type: "recv_pass", to: "west", cards: ["7D", "5D", "3D"] },
  //   { type: "charge", seat: "north", cards: ["QS"] },
  //   { type: "charge", seat: "east", cards: ["AH"] },
  //   { type: "charge", seat: "west", cards: ["JD"] },
  //   { type: "charge", seat: "north", cards: [] },
  //   { type: "charge", seat: "east", cards: [] }
];
