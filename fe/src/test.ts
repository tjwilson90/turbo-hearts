export const TEST_EVENTS = [
  {
    type: "sit",
    north: { type: "bot", name: "gdawson (bot)", algorithm: "random" },
    east: { type: "bot", name: "nhardin (bot)", algorithm: "random" },
    south: { type: "human", name: "dan" },
    west: { type: "bot", name: "mlove (bot)", algorithm: "random" },
    rules: "classic"
  },
  {
    type: "deal",
    north: ["2C", "4C", "5C", "6C", "7C", "8C", "9C", "TC", "JC", "QC", "KC", "AC", "AH"],
    east: ["2D", "3D", "4D", "5D", "6D", "7D", "8D", "9D", "TD", "JD", "QD", "KD", "AD"],
    south: ["2H", "3H", "4H", "5H", "6H", "7H", "8H", "9H", "TH", "JH", "QH", "KH", "3C"],
    west: ["2S", "3S", "4S", "5S", "6S", "7S", "8S", "9S", "TS", "JS", "QS", "KS", "AS"],
    pass: "left"
  },
  { type: "start_passing" },
  { type: "send_pass", from: "north", cards: ["2C", "8C", "KC"] },
  { type: "send_pass", from: "west", cards: ["2S", "8S", "KS"] },
  { type: "recv_pass", to: "north", cards: ["4S", "4H", "6D"] },
  { type: "send_pass", from: "east", cards: ["2D", "8D", "KD"] },
  { type: "recv_pass", to: "east", cards: ["2H", "8D", "QC"] },
  { type: "send_pass", from: "south", cards: ["2H", "8H", "KH"] },
  { type: "recv_pass", to: "south", cards: ["2C", "8C", "KC"] },
  { type: "recv_pass", to: "west", cards: ["7D", "5D", "3D"] },
  { type: "charge", seat: "north", cards: ["TC", "AH"] },
  { type: "charge", seat: "east", cards: ["JD"] },
  { type: "charge", seat: "west", cards: ["QS"] },
  { type: "charge", seat: "north", cards: [] },
  { type: "charge", seat: "east", cards: [] }
];
