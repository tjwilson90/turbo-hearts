import * as React from "react";
import { TurboHeartsLobbyService } from "../TurboHeartsLobbyService";
import { ChatState, GamesState, LobbyState, UsersState } from "../state/types";
import { connect } from "react-redux";
import { GameListItem } from "./GameListItem";
import { Dispatch } from "redux";
import { ToggleHideOldGames } from "../state/actions";
import { MessageItem } from "./MessageItem";
import { LeagueScores } from "./LeagueScores";

export namespace Lobby {
    export interface OwnProps {
        service: TurboHeartsLobbyService;
    }

    export interface StoreProps {
        users: UsersState;
        games: GamesState;
        chat: ChatState;
        hideOldGames: boolean;
    }

    export interface DispatchProps {
        toggleHideOlderGames(): void;
        createNewGame(): void;
        onChat(msg: string): void;
    }

    export type Props = OwnProps & StoreProps & DispatchProps;
}

function mapStateToProps(state: LobbyState): Lobby.StoreProps {
    return {
        chat: state.chats.lobby,
        games: state.games,
        users: state.users,
        hideOldGames: state.ui.hideOldGames
    };
}

function mapDispatchToProps(dispatch: Dispatch, ownProps: Lobby.OwnProps) {
    return {
        toggleHideOlderGames() {
            dispatch(ToggleHideOldGames());
        },
        createNewGame() {
            ownProps.service.createLobby("classic");
        },
        onChat(msg: string): void {
            ownProps.service.chat(msg);
        }
    };
}

class LobbyInternal extends React.PureComponent<Lobby.Props> {
    private inputRef: HTMLTextAreaElement | null = null;
    private scrollRef: HTMLDivElement | null = null;

    public render() {
        return (
            <div className="lobby-wrapper">
                <div className="game-list">
                    <div className="header">
                        <div className="game-list-item -selected">Lobby</div>
                    </div>
                    <div className="list">{this.renderGamesList()}</div>
                    <div className="footer">
                        <div className="button-group">
                            <div className="button" onClick={this.props.createNewGame}>
                                New game
                            </div>
                            <div className="button" onClick={this.props.toggleHideOlderGames}>
                                {this.props.hideOldGames ? "Show older games" : "Hide older games"}
                            </div>
                        </div>
                    </div>
                </div>
                <div className="message-list">
                    <div className="list" onClick={this.focusTextInput} ref={el => (this.scrollRef = el)}>
                        {this.renderMessages()}
                    </div>
                    <div className="entry">{this.renderChatInput()}</div>
                </div>
                <div className="league-scores">
                    <LeagueScores />
                </div>
                <div className="user-list">{this.renderUserList()}</div>
            </div>
        );
    }

    public componentDidUpdate(prevProps: Lobby.Props) {
        if (prevProps.chat.messages.length !== this.props.chat.messages.length && this.scrollRef != null) {
            this.scrollRef.scrollTop = this.scrollRef.scrollHeight;
        }
    }

    private renderGamesList() {
        let sortedGames = Object.values(this.props.games);

        sortedGames.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

        if (this.props.hideOldGames) {
            sortedGames = sortedGames.filter(game => Date.now() - 1000 * 60 * 10 < game.updatedAt.getTime());
        }

        return sortedGames.map(game => <GameListItem game={game} service={this.props.service} />);
    }

    private renderMessages() {
        return this.props.chat.messages.map((message, index) => <MessageItem key={index} message={message} />);
    }

    private renderChatInput() {
        return (
            <textarea
                className="chat-input"
                placeholder="Enter chat message..."
                onKeyPress={this.handleKeyPress}
                autoFocus={true}
                ref={el => (this.inputRef = el)}
            />
        );
    }

    private renderUserList() {
        const users = this.props.chat.userIds
            .map(userId => ({
                userId,
                userName: this.props.users.userNamesByUserId[userId]
            }))
            .sort((a, b) => {
                if (a.userName === undefined) {
                    return 1;
                }
                if (b.userName === undefined) {
                    return -1;
                }
                return a.userName.localeCompare(b.userName);
            });

        return users.map(({ userId, userName }) => (
            <div className="user-name-item" key={userId}>
                {userName || userId}
            </div>
        ));
    }

    private focusTextInput = () => {
        if (this.inputRef != null) {
            this.inputRef.focus();
        }
    };

    private handleKeyPress = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (event.key === "Enter" && event.currentTarget.value.trim().length !== 0) {
            this.props.onChat(event.currentTarget.value);
            event.currentTarget.value = "";
            event.preventDefault();
            event.stopPropagation();
        }
    };
}

export const Lobby = connect(mapStateToProps, mapDispatchToProps)(LobbyInternal);
