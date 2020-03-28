import * as React from "react";
import { ChatMessage, ISubstitutions, LobbyState, UsersState } from "../state/types";
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
                    </span>&nbsp;{this.renderMessage(msg.message, msg.substitutions)}
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

    private renderMessage(msg: string, subs: ISubstitutions[]) {
        const consumed: JSX.Element[] = [];
        let unconsumed = msg;

        for (let i = 0; i < subs.length; i++) {
            const searchString = "$" + i;
            const subIdx = unconsumed.indexOf(searchString);

            consumed.push(<React.Fragment key={consumed.length}>{unconsumed.slice(0, subIdx)}</React.Fragment>);

            const sub = subs[i];
            if (sub.type === "game") {
                consumed.push(<a key={consumed.length} className="inline-message-link" href={`/game#${sub.gameId}`} target="_blank">Open game</a>);
            } else if (sub.type === "user") {
                consumed.push(<React.Fragment key={consumed.length}>{this.props.users.userNamesByUserId[sub.userId] || "Loading"}</React.Fragment>);
            } else if (sub.type === "bot") {
                consumed.push(<React.Fragment key={consumed.length}>{sub.strategy.slice(0, 1).toUpperCase() + sub.strategy.slice(1)}</React.Fragment>);
            }

            unconsumed = unconsumed.slice(subIdx + searchString.length);
        }
        consumed.push(<React.Fragment key={consumed.length}>{unconsumed}</React.Fragment>);
        return consumed;
    }

}

export const MessageItem = connect(mapStateToProps)(MessageItemInternal);
