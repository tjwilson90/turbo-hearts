import * as React from "react";
import { ChatMessage, LobbyState, UsersState } from "../state/types";
import { connect } from "react-redux";

export namespace MessageItem {
    export interface OwnProps {
        message: ChatMessage;
    }

    export interface StoreProps {
        users: UsersState;
    }

    export type Props = OwnProps & StoreProps;
}

function mapStateToProps(state: LobbyState): MessageItem.StoreProps {
    return {
        users: state.users,
    }
}


class MessageItemInternal extends React.PureComponent<MessageItem.Props> {
    public render() {
        const msg = this.props.message;
        const hours = msg.date.getHours() < 10 ? "0" + msg.date.getHours() : msg.date.getHours();
        const minutes = msg.date.getMinutes() < 10 ? "0" + msg.date.getMinutes() : msg.date.getMinutes();
        return (
            <div className="message-item">
                <div className="time">{hours}:{minutes}</div>
                <div className="message">
                    <span className="user-name">
                        {this.props.users.userNamesByUserId[msg.userId] !== undefined
                            ? this.props.users.userNamesByUserId[msg.userId]
                            : "Loading"}
                    </span>&nbsp;{msg.message}
                </div>
            </div>
        );
    }

}

export const MessageItem = connect(mapStateToProps)(MessageItemInternal);
