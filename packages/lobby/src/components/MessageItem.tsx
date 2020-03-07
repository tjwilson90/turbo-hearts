import * as React from "react";
import { ChatMessage, LobbyState, UsersState } from "../state/types";
import { connect } from "react-redux";
import classNames from "classnames";

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
                <div className={classNames("message", {"-generated": msg.generated})}>
                    <span className="user-name">
                        {this.renderUserName(msg.userId)}
                    </span>&nbsp;{this.renderMessage(msg.message)}
                </div>
            </div>
        );
    }

    private renderUserName(userId: string | undefined) {
        if (userId === undefined) {
            return null;
        }
        return this.props.users.userNamesByUserId[userId] !== undefined
            ? this.props.users.userNamesByUserId[userId]
            : "Loading"
    }

    private renderMessage(msg: string) {
        const consumed: JSX.Element[] = [];
        let unconsumed = msg;
        while (unconsumed.length > 0) {
            const gameLinkIndex = unconsumed.indexOf("$gameUrl=");
            if (gameLinkIndex !== -1) {
                const gameHash = unconsumed.substr(gameLinkIndex + 9, 36);
                consumed.push(<React.Fragment key={consumed.length}>{unconsumed.substr(0, gameLinkIndex)}</React.Fragment>);
                consumed.push(<a key={consumed.length} className="inline-message-link" href={`/game#${gameHash}`} target="_blank">Open game</a>);
                unconsumed = msg.slice(gameLinkIndex + 45);
                continue;
            }
            consumed.push(<React.Fragment key={consumed.length}>{unconsumed}</React.Fragment>);
            unconsumed = "";
        }
        return consumed;
    }

}

export const MessageItem = connect(mapStateToProps)(MessageItemInternal);
