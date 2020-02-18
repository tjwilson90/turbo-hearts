import { EventData } from "./types";

export const HIDDEN_GAME: EventData[] = [
  {
    type: "sit",
    north: { type: "bot", name: "qyork (bot)", algorithm: "random" },
    east: { type: "bot", name: "ypotts (bot)", algorithm: "random" },
    south: { type: "bot", name: "lpreston (bot)", algorithm: "random" },
    west: { type: "human", name: "dan" },
    rules: "classic"
  },
  {
    type: "deal",
    north: [],
    east: [],
    south: [],
    west: ["JS", "TS", "6S", "AH", "KH", "TH", "9H", "4H", "QD", "8D", "8C", "5C", "4C"],
    pass: "left"
  },
  { type: "send_pass", from: "north", cards: [] },
  { type: "send_pass", from: "south", cards: [] },
  { type: "send_pass", from: "east", cards: [] },
  { type: "recv_pass", to: "east", cards: [] },
  { type: "recv_pass", to: "south", cards: [] },
  { type: "send_pass", from: "west", cards: ["TS", "QD", "8D"] },
  { type: "recv_pass", to: "west", cards: ["AS", "9S", "6C"] },
  { type: "recv_pass", to: "north", cards: [] },
  { type: "charge", seat: "east", cards: ["JD"] },
  { type: "charge", seat: "south", cards: [] },
  { type: "charge", seat: "north", cards: ["TC"] },
  { type: "charge", seat: "east", cards: [] },
  { type: "charge", seat: "south", cards: [] },
  { type: "charge", seat: "west", cards: [] },
  { type: "start_trick", leader: "south" },
  { type: "play", seat: "south", card: "2C" }
  //   { type: "play", seat: "west", card: "8C" },
  //   { type: "play", seat: "north", card: "AC" },
  //   { type: "play", seat: "east", card: "3C" },
  //   { type: "end_trick", winner: "north" },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "TC" },
  //   { type: "play", seat: "east", card: "KC" },
  //   { type: "play", seat: "south", card: "7C" },
  //   { type: "play", seat: "west", card: "5C" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "TD" },
  //   { type: "play", seat: "south", card: "5D" },
  //   { type: "play", seat: "west", card: "6C" },
  //   { type: "play", seat: "north", card: "2D" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "5S" },
  //   { type: "play", seat: "south", card: "3S" },
  //   { type: "play", seat: "west", card: "9S" },
  //   { type: "play", seat: "north", card: "TS" },
  //   { type: "play", seat: "east", card: "2S" },
  //   { type: "play", seat: "south", card: "8S" },
  //   { type: "play", seat: "west", card: "6S" },
  //   { type: "play", seat: "north", card: "7H" },
  //   { type: "end_trick", winner: "north" },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "3D" },
  //   { type: "play", seat: "east", card: "JD" },
  //   { type: "play", seat: "south", card: "6D" },
  //   { type: "play", seat: "west", card: "JS" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "5H" },
  //   { type: "play", seat: "south", card: "8H" },
  //   { type: "play", seat: "west", card: "9H" },
  //   { type: "play", seat: "north", card: "QH" },
  //   { type: "play", seat: "east", card: "6H" },
  //   { type: "play", seat: "south", card: "9C" },
  //   { type: "play", seat: "west", card: "KH" },
  //   { type: "play", seat: "north", card: "2H" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "AH" },
  //   { type: "play", seat: "north", card: "JH" },
  //   { type: "play", seat: "east", card: "3H" },
  //   { type: "play", seat: "south", card: "4D" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "TH" },
  //   { type: "play", seat: "north", card: "8D" },
  //   { type: "play", seat: "east", card: "QC" },
  //   { type: "play", seat: "south", card: "JC" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "4H" },
  //   { type: "play", seat: "north", card: "QD" },
  //   { type: "play", seat: "east", card: "KD" },
  //   { type: "play", seat: "south", card: "7S" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "AS" },
  //   { type: "play", seat: "north", card: "AD" },
  //   { type: "play", seat: "east", card: "QS" },
  //   { type: "play", seat: "south", card: "4S" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "4C" },
  //   { type: "play", seat: "north", card: "9D" },
  //   { type: "play", seat: "east", card: "7D" },
  //   { type: "play", seat: "south", card: "KS" },
  //   { type: "end_trick", winner: "west" },
  //   {
  //     type: "deal",
  //     north: [],
  //     east: [],
  //     south: [],
  //     west: ["KS", "JS", "TS", "6S", "3S", "2S", "AH", "KH", "6H", "3H", "KC", "8C", "2C"],
  //     pass: "right"
  //   },
  //   { type: "send_pass", from: "south", cards: [] },
  //   { type: "send_pass", from: "north", cards: [] },
  //   { type: "send_pass", from: "east", cards: [] },
  //   { type: "recv_pass", to: "east", cards: [] },
  //   { type: "recv_pass", to: "north", cards: [] },
  //   { type: "send_pass", from: "west", cards: ["KC", "8C", "2C"] },
  //   { type: "recv_pass", to: "west", cards: ["QS", "7S", "JC"] },
  //   { type: "recv_pass", to: "south", cards: [] },
  //   { type: "charge", seat: "north", cards: [] },
  //   { type: "charge", seat: "south", cards: [] },
  //   { type: "charge", seat: "east", cards: ["TC"] },
  //   { type: "charge", seat: "north", cards: [] },
  //   { type: "charge", seat: "south", cards: [] },
  //   { type: "charge", seat: "west", cards: ["QS"] },
  //   { type: "charge", seat: "north", cards: [] },
  //   { type: "charge", seat: "east", cards: [] },
  //   { type: "charge", seat: "south", cards: [] },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "2C" },
  //   { type: "play", seat: "west", card: "JC" },
  //   { type: "play", seat: "north", card: "6C" },
  //   { type: "play", seat: "east", card: "TC" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "TS" },
  //   { type: "play", seat: "north", card: "9S" },
  //   { type: "play", seat: "east", card: "5S" },
  //   { type: "play", seat: "south", card: "TD" },
  //   { type: "play", seat: "west", card: "JS" },
  //   { type: "play", seat: "north", card: "8S" },
  //   { type: "play", seat: "east", card: "8H" },
  //   { type: "play", seat: "south", card: "KC" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "KH" },
  //   { type: "play", seat: "north", card: "5H" },
  //   { type: "play", seat: "east", card: "QH" },
  //   { type: "play", seat: "south", card: "3C" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "AH" },
  //   { type: "play", seat: "north", card: "2H" },
  //   { type: "play", seat: "east", card: "JH" },
  //   { type: "play", seat: "south", card: "9D" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "6H" },
  //   { type: "play", seat: "north", card: "4H" },
  //   { type: "play", seat: "east", card: "7H" },
  //   { type: "play", seat: "south", card: "JD" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "9C" },
  //   { type: "play", seat: "south", card: "AC" },
  //   { type: "play", seat: "west", card: "QS" },
  //   { type: "play", seat: "north", card: "4S" },
  //   { type: "play", seat: "east", card: "4C" },
  //   { type: "play", seat: "south", card: "QC" },
  //   { type: "play", seat: "west", card: "3H" },
  //   { type: "play", seat: "north", card: "2D" },
  //   { type: "end_trick", winner: "south" },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "8C" },
  //   { type: "play", seat: "west", card: "KS" },
  //   { type: "play", seat: "north", card: "6D" },
  //   { type: "play", seat: "east", card: "7C" },
  //   { type: "end_trick", winner: "south" },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "4D" },
  //   { type: "play", seat: "west", card: "6S" },
  //   { type: "play", seat: "north", card: "3D" },
  //   { type: "play", seat: "east", card: "AD" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "TH" },
  //   { type: "play", seat: "south", card: "5D" },
  //   { type: "play", seat: "west", card: "7S" },
  //   { type: "play", seat: "north", card: "9H" },
  //   { type: "play", seat: "east", card: "QD" },
  //   { type: "play", seat: "south", card: "8D" },
  //   { type: "play", seat: "west", card: "3S" },
  //   { type: "play", seat: "north", card: "AS" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "5C" },
  //   { type: "play", seat: "south", card: "KD" },
  //   { type: "play", seat: "west", card: "2S" },
  //   { type: "play", seat: "north", card: "7D" },
  //   { type: "end_trick", winner: "east" },
  //   {
  //     type: "deal",
  //     north: [],
  //     east: [],
  //     south: [],
  //     west: ["3S", "QH", "JH", "TH", "7H", "5H", "2H", "4D", "2D", "9C", "8C", "6C", "5C"],
  //     pass: "across"
  //   },
  //   { type: "send_pass", from: "north", cards: [] },
  //   { type: "send_pass", from: "east", cards: [] },
  //   { type: "send_pass", from: "south", cards: [] },
  //   { type: "recv_pass", to: "south", cards: [] },
  //   { type: "recv_pass", to: "north", cards: [] },
  //   { type: "send_pass", from: "west", cards: ["QH", "4D", "2D"] },
  //   { type: "recv_pass", to: "west", cards: ["AS", "KC", "QC"] },
  //   { type: "recv_pass", to: "east", cards: [] },
  //   { type: "charge", seat: "east", cards: [] },
  //   { type: "charge", seat: "south", cards: ["AH", "JD"] },
  //   { type: "charge", seat: "north", cards: ["TC"] },
  //   { type: "charge", seat: "east", cards: [] },
  //   { type: "charge", seat: "south", cards: [] },
  //   { type: "charge", seat: "west", cards: [] },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "2C" },
  //   { type: "play", seat: "east", card: "4C" },
  //   { type: "play", seat: "south", card: "AC" },
  //   { type: "play", seat: "west", card: "9C" },
  //   { type: "play", seat: "north", card: "TC" },
  //   { type: "play", seat: "east", card: "2D" },
  //   { type: "play", seat: "south", card: "JD" },
  //   { type: "play", seat: "west", card: "KC" },
  //   { type: "end_trick", winner: "south" },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "8S" },
  //   { type: "play", seat: "west", card: "AS" },
  //   { type: "play", seat: "north", card: "6S" },
  //   { type: "play", seat: "east", card: "JS" },
  //   { type: "end_trick", winner: "west" },
  //   { type: "start_trick", leader: "west" },
  //   { type: "play", seat: "west", card: "3S" },
  //   { type: "play", seat: "north", card: "4S" },
  //   { type: "play", seat: "east", card: "KS" },
  //   { type: "play", seat: "south", card: "9S" },
  //   { type: "play", seat: "west", card: "QC" },
  //   { type: "play", seat: "north", card: "TS" },
  //   { type: "play", seat: "east", card: "4D" },
  //   { type: "play", seat: "south", card: "QS" },
  //   { type: "end_trick", winner: "east" },
  //   { type: "start_trick", leader: "east" },
  //   { type: "play", seat: "east", card: "7D" },
  //   { type: "play", seat: "south", card: "TD" },
  //   { type: "play", seat: "west", card: "JH" },
  //   { type: "play", seat: "north", card: "9D" },
  //   { type: "play", seat: "east", card: "6D" },
  //   { type: "play", seat: "south", card: "KD" },
  //   { type: "play", seat: "west", card: "TH" },
  //   { type: "play", seat: "north", card: "3C" },
  //   { type: "end_trick", winner: "south" },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "2S" },
  //   { type: "play", seat: "west", card: "7H" },
  //   { type: "play", seat: "north", card: "7S" },
  //   { type: "play", seat: "east", card: "QH" },
  //   { type: "end_trick", winner: "north" },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "7C" },
  //   { type: "play", seat: "east", card: "5D" },
  //   { type: "play", seat: "south", card: "AH" },
  //   { type: "play", seat: "west", card: "6C" },
  //   { type: "end_trick", winner: "north" },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "4H" },
  //   { type: "play", seat: "east", card: "6H" },
  //   { type: "play", seat: "south", card: "8H" },
  //   { type: "play", seat: "west", card: "5H" },
  //   { type: "end_trick", winner: "south" },
  //   { type: "start_trick", leader: "south" },
  //   { type: "play", seat: "south", card: "9H" },
  //   { type: "play", seat: "west", card: "2H" },
  //   { type: "play", seat: "north", card: "KH" },
  //   { type: "play", seat: "east", card: "3H" },
  //   { type: "play", seat: "south", card: "QD" },
  //   { type: "play", seat: "west", card: "8C" },
  //   { type: "play", seat: "north", card: "JC" },
  //   { type: "play", seat: "east", card: "8D" },
  //   { type: "end_trick", winner: "north" },
  //   { type: "start_trick", leader: "north" },
  //   { type: "play", seat: "north", card: "5S" },
  //   { type: "play", seat: "east", card: "3D" },
  //   { type: "play", seat: "south", card: "AD" },
  //   { type: "play", seat: "west", card: "5C" },
  //   { type: "end_trick", winner: "north" },
  //   {
  //     type: "deal",
  //     north: [],
  //     east: [],
  //     south: [],
  //     west: ["TS", "9S", "AH", "KH", "6H", "4H", "3H", "KD", "TD", "8D", "9C", "7C", "5C"],
  //     pass: "keeper"
  //   },
  //   { type: "charge", seat: "north", cards: [] },
  //   { type: "charge", seat: "east", cards: ["QS"] },
  //   { type: "charge", seat: "south", cards: [] },
  //   { type: "charge", seat: "north", cards: [] }
];
