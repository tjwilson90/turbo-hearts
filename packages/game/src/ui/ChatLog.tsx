import * as React from "react";
import { connect } from "react-redux";
import { ChatMessage, GameAppState, User } from "../state/types";
import { Suit, colorMap, suitMap } from "./PlayHistory";
import classNames from "classnames";
import { GAME_BOT, Seat } from "../types";
import { POSITIONS } from "../util/seatPositions";

export namespace ChatLog {
  export interface StoreProps {
    messages: ChatMessage[];
    users: { [key: string]: User };
    top: User | undefined;
    right: User | undefined;
    bottom: User | undefined;
    left: User | undefined;
    bottomSeat: Seat;
  }

  export type Props = StoreProps;
}

type Rank = "A" | "K" | "Q" | "J" | "T" | "9" | "8" | "7" | "6" | "5" | "4" | "3" | "2" | "X" | "x";

const sortOrder: { [key in Rank]: number } = {
  A: 0,
  K: 1,
  Q: 2,
  J: 3,
  T: 4,
  "9": 5,
  "8": 6,
  "7": 7,
  "6": 8,
  "5": 9,
  "4": 10,
  "3": 11,
  "2": 12,
  X: 13,
  x: 13
};

function ActualCardRun(props: { cardRun: string }) {
  let { cardRun } = props;
  const suit = cardRun.substring(cardRun.length - 1) as Suit;
  const cards: Rank[] = [];
  for (let i = 0; i < cardRun.length - 1; i++) {
    let rank = cardRun[i] as Rank;
    cards.push(rank);
  }
  cards.sort((a, b) => sortOrder[a] - sortOrder[b]);
  const renderCards = cards.map(c => {
    return c === "T" && cards.length === 1 ? "10" : c === "X" ? "x" : c;
  });
  return <span style={{ color: colorMap[suit], fontWeight: 700 }}>{`${renderCards.join("")}${suitMap[suit]}`}</span>;
}

function isSuit(s: string): s is Suit {
  return s === "C" || s === "D" || s === "H" || s === "S";
}

function cardRuns(cardRun: string) {
  if (cardRun === "Q") {
    cardRun = "QS";
  } else if (cardRun === "J") {
    cardRun = "JD";
  } else if (cardRun === "T") {
    cardRun = "TC";
  }
  const runs: string[] = [];
  let start = 0;
  for (let i = 0; i < cardRun.length; i++) {
    let letter = cardRun[i];
    if (isSuit(letter)) {
      runs.push(cardRun.substring(start, i + 1));
      start = i + 1;
    }
  }
  return runs.map(run => <ActualCardRun cardRun={run} />);
}

class ChatLogInternal extends React.Component<ChatLog.Props> {
  private ref = React.createRef<HTMLDivElement>();

  public render() {
    if (this.props.bottomSeat === undefined) {
      return <div className="chat-log" ref={this.ref} />;
    }
    return (
      <div className="chat-log" ref={this.ref}>
        {this.props.messages.map(this.renderMessage)}
      </div>
    );
  }

  public componentDidUpdate(prevProps: ChatLog.Props) {
    if (prevProps.messages.length !== this.props.messages.length && this.ref.current != null) {
      this.ref.current.scrollTop = this.ref.current.scrollHeight;
    }
  }

  private renderMessage = (message: ChatMessage, idx: number) => {
    const user = this.props.users[message.userId];
    const chunks = this.convertToRichCardText(this.substituteDirections(message.message));
    return (
      <div className={classNames("chat-message-container", { "game-bot": message.userId === GAME_BOT })} key={idx}>
        <span className="chat-user">{user?.name ?? "loading..."}</span>
        <span className="chat-message">{chunks}</span>
      </div>
    );
  };

  private substituteDirections(message: string) {
    const positions = POSITIONS[this.props.bottomSeat];
    return message
      .replace(/__north/g, this.props[positions.north]?.name ?? "north")
      .replace(/__east/g, this.props[positions.east]?.name ?? "east")
      .replace(/__south/g, this.props[positions.south]?.name ?? "south")
      .replace(/__west/g, this.props[positions.west]?.name ?? "west")
      .replace(/__([0-9a-fA-F]{8}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{4}\-[0-9a-fA-F]{12})/g, (_match, p1) => this.props.users[p1]?.name ?? "User");
  }

  private convertToRichCardText(message: string) {
    const cardRegex = /\b((T)|(J)|(QT)|(JT)|(Q)|([23456789TJQKAxX]+[CDHS]))+\b/g;
    let result;
    let last = 0;
    const chunks = [];
    while ((result = cardRegex.exec(message))) {
      const sub = message.substring(last, result.index);
      chunks.push(sub);
      chunks.push(...cardRuns(result[0]));
      last = result.index + result[0].length;
    }
    chunks.push(message.substring(last));
    return chunks;
  }
}

function mapStateToProps(state: GameAppState): ChatLog.StoreProps {
  return {
    users: state.users.users,
    messages: state.chat.messages,
    ...state.game
  };
}

export const ChatLog = connect(mapStateToProps)(ChatLogInternal);
