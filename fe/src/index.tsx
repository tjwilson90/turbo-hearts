import "./styles/style.scss";
import * as ReactDOM from "react-dom";
import * as React from "react";
import { Hand } from "./ui/Hand";
import { Card } from "./types";
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

document.addEventListener("DOMContentLoaded", event => {
  //   subscribe();
  const SAMPLE_HAND: Card[] = [
    "2C",
    "3C",
    "7C",
    "9C",
    "TC",
    "4D",
    "JD",
    "AD",
    "KH",
    "3S",
    "5S",
    "QS",
    "KS"
  ];

  const HIDDEN_HAND: Card[] = [
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
  ReactDOM.render(
    <div>
      <Table
        north={HIDDEN_HAND}
        east={HIDDEN_HAND}
        south={SAMPLE_HAND}
        west={HIDDEN_HAND}
      ></Table>
    </div>,
    document.body
  );
});
