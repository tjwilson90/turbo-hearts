import * as React from "react";
import { connect } from "react-redux";
import { ChatMessage, GameAppState, User } from "../state/types";

export namespace ChatLog {
  export interface StoreProps {
    messages: ChatMessage[];
    users: { [key: string]: User };
  }

  export type Props = StoreProps;
}

class ChatLogInternal extends React.Component<ChatLog.Props> {
  private canvasRef = React.createRef<HTMLCanvasElement>();

  public render() {
    return <div className="chat-log">{this.props.messages.map(this.renderMessage)}</div>;
  }

  private renderMessage = (message: ChatMessage, idx: number) => {
    const user = this.props.users[message.userId];
    return (
      <div className="chat-message-container" key={idx}>
        <span className="chat-user">{user?.name ?? "loading..."}</span>
        <span className="chat-message">{message.message}</span>
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
