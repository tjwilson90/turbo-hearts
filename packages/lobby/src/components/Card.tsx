import * as React from "react";

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

export type Suit = "S" | "H" | "D" | "C";
export const suitMap: { [key in Suit]: string } = {
    S: "♠",
    H: "♥",
    D: "♦",
    C: "♣",
};

export const colorMap: { [key in Suit]: string } = {
    S: "#000000",
    H: "#ff0000",
    D: "#0d00ff",
    C: "#008000",
};

export function NiceCard(props: { card: Card }) {
    if (props.card == null) {
        return null;
    }
    let rank = props.card.substring(0, 1);
    if (rank === "T") {
        rank = "10";
    }
    const suit = props.card.substring(1) as Suit;

    return <span style={{ color: colorMap[suit] }}>{`${rank}${suitMap[suit]}`}</span>;
}

export function NiceSuit(props: { suit: Suit }) {
    if (props.suit == null) {
        return null;
    }
    return <span style={{ color: colorMap[props.suit] }}>{`${suitMap[props.suit]}`}</span>;
}
