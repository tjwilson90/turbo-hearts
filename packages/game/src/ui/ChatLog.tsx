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
