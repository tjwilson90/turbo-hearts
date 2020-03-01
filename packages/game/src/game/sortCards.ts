import { SpriteCard, Card } from "../types";

const sortOrder: { [key: string]: number } = {
  BACK: 0,
  "2C": 1,
  "3C": 2,
  "4C": 3,
  "5C": 4,
  "6C": 5,
  "7C": 6,
  "8C": 7,
  "9C": 8,
  TC: 9,
  JC: 10,
  QC: 11,
  KC: 12,
  AC: 13,
  "2D": 14,
  "3D": 15,
  "4D": 16,
  "5D": 17,
  "6D": 18,
  "7D": 19,
  "8D": 20,
  "9D": 21,
  TD: 22,
  JD: 23,
  QD: 24,
  KD: 25,
  AD: 26,
  "2H": 27,
  "3H": 28,
  "4H": 29,
  "5H": 30,
  "6H": 31,
  "7H": 32,
  "8H": 33,
  "9H": 34,
  TH: 35,
  JH: 36,
  QH: 37,
  KH: 38,
  AH: 39,
  "2S": 40,
  "3S": 41,
  "4S": 42,
  "5S": 43,
  "6S": 44,
  "7S": 45,
  "8S": 46,
  "9S": 47,
  TS: 48,
  JS: 49,
  QS: 50,
  KS: 51,
  AS: 52
};

export function sortCards(cards: Card[]) {
  cards.sort((a, b) => {
    return sortOrder[a] - sortOrder[b];
  });
}

export function sortSpriteCards(cards: SpriteCard[]) {
  cards.sort((a, b) => {
    return sortOrder[a.card] - sortOrder[b.card];
  });
}
