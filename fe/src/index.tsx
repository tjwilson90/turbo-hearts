import "./styles/style.scss";
import * as ReactDOM from "react-dom";
import * as React from "react";
import { Hand } from "./ui/Hand";
import { Card, Player } from "./types";
import { Table } from "./ui/Table";

const SERVER_URL = "https://localhost:7380/lobby/subscribe";
let eventStream: EventSource | null = null;

function subscribe() {
  if (eventStream != null) {
    eventStream.close();
  }
  eventStream = new EventSource(SERVER_URL);
  eventStream.onmessage = event => {
    console.log(event.data);
  };
}

const SAMPLE_HAND: Card[] = [
  "2C",
  "3C",
  "7C",
  "8C",
  "JC",
  "4D",
  "JD",
  "AD",
  "KH",
  "3S",
  "5S",
  "QS",
  "KS"
];

const HIDDEN_HAND_1: Card[] = [
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "JD"
];

const HIDDEN_HAND_2: Card[] = [
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK"
];

const HIDDEN_HAND_3: Card[] = [
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "BACK",
  "AH",
  "TC"
];

const DAN: Player = {
  name: "Dan",
  chargedAh: false,
  chargedJd: false,
  chargedTc: false,
  chargedQs: true,
  dealer: true,
  hand: SAMPLE_HAND
};

const HIDDEN_1: Player = {
  name: "Tim S",
  chargedAh: false,
  chargedJd: true,
  chargedTc: false,
  chargedQs: false,
  dealer: false,
  hand: HIDDEN_HAND_1
};

const HIDDEN_2: Player = {
  name: "John C",
  chargedAh: false,
  chargedJd: false,
  chargedTc: false,
  chargedQs: false,
  dealer: false,
  hand: HIDDEN_HAND_2
};

const HIDDEN_3: Player = {
  name: "Tim W",
  chargedAh: true,
  chargedJd: false,
  chargedTc: true,
  chargedQs: false,
  dealer: false,
  hand: HIDDEN_HAND_3
};

document.addEventListener("DOMContentLoaded", event => {
  //   subscribe();

  ReactDOM.render(
    <div className="turbo-hearts-container">
      <Table
        top={HIDDEN_1}
        topPlays={["2C", "7C"]}
        right={HIDDEN_2}
        rightPlays={["9C", "8C"]}
        bottom={DAN}
        bottomPlays={["3C", "8C"]}
        left={HIDDEN_3}
        leftPlays={["3C", "8C"]}
      ></Table>
    </div>,
    document.body
  );
});
