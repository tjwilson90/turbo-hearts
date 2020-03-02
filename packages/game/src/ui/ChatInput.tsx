import * as React from "react";

export namespace ChatInput {
  export interface Props {
    onChat: (message: string) => void;
  }
  export interface State {
    text: string;
  }
}

export class ChatInput extends React.Component<ChatInput.Props, ChatInput.State> {
  public state: ChatInput.State = {
    text: ""
  };

  public render() {
    return (
      <textarea
        value={this.state.text}
        className="chat-input"
        placeholder="Enter chat message..."
        onChange={this.handleChange}
        onKeyPress={this.handleKeyPress}
      ></textarea>
    );
  }

  private handleChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    this.setState({ text: event.target.value });
  };

  private handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === "Enter") {
      this.props.onChat(this.state.text);
      this.setState({ text: "" });
      event.preventDefault();
      event.stopPropagation();
    }
  };
}
