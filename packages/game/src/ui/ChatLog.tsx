import * as React from "react";
import { connect } from "react-redux";
import { ChatMessage, GameAppState, User } from "../state/types";
import { Suit, colorMap, suitMap } from "./PlayHistory";

export namespace ChatLog {
  export interface StoreProps {
    messages: ChatMessage[];
    users: { [key: string]: User };
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

function NiceCardRun(props: { cardRun: string }) {
  const { cardRun } = props;
  if (cardRun == null) {
    return null;
  }
  const suit = cardRun.substring(cardRun.length - 1) as Suit;
  const cards: Rank[] = [];
  for (let i = 0; i < cardRun.length - 1; i++) {
    let rank = cardRun[i] as Rank;
    cards.push(rank);
  }
  cards.sort((a, b) => sortOrder[a] - sortOrder[b]);
  const renderCards = cards.map(c => (c === "T" ? "10" : c === "X" ? "x" : c));
  return <span style={{ color: colorMap[suit], fontWeight: 700 }}>{`${renderCards.join("")}${suitMap[suit]}`}</span>;
}

class ChatLogInternal extends React.Component<ChatLog.Props> {
  private ref = React.createRef<HTMLDivElement>();

  public render() {
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
    const cardRegex = /\b([23456789TJQKAxX]+[CDHS])\b/g;
    let result;
    let last = 0;
    const chunks = [];
    while ((result = cardRegex.exec(message.message))) {
      const sub = message.message.substring(last, result.index);
      chunks.push(sub);
      chunks.push(<NiceCardRun cardRun={result[0]} />);
      last = result.index + result[0].length;
    }
    chunks.push(message.message.substring(last));
    return (
      <div className="chat-message-container" key={idx}>
        <span className="chat-user">{user?.name ?? "loading..."}</span>
        <span className="chat-message">{chunks}</span>
      </div>
    );
  };
}

function mapStateToProps(state: GameAppState): ChatLog.StoreProps {
  return {
    users: state.users.users,
    messages: state.chat.messages
  };
}

export const ChatLog = connect(mapStateToProps)(ChatLogInternal);
