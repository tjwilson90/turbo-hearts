import * as React from "react";
import { TurboHeartsLobbyService } from "../TurboHeartsLobbyService";
import classNames from "classnames";
import { LobbyGame, LobbyState, UsersState } from "../state/types";
import { connect } from "react-redux";
import { Dispatch } from "redux";
import { BotStrategy, LobbyPlayer } from "../types";

export namespace GameListItem {
    export interface OwnProps {
        game: LobbyGame;
        service: TurboHeartsLobbyService;
    }

    export interface StoreProps {
        userId: string;
        users: UsersState;
    }

    export interface DispatchProps {
        joinGame(): void;

        leaveGame(): void;

        addBot(strategy: BotStrategy): void;
    }

    export type Props = OwnProps & StoreProps & DispatchProps;
}

function mapStateToProps(state: LobbyState): GameListItem.StoreProps {
    return {
        users: state.users,
        userId: state.users.ownUserId,
    }
}

function mapDispatchToProps(_dispatch: Dispatch, ownProps: GameListItem.OwnProps): GameListItem.DispatchProps {
    return {
        addBot(strategy: BotStrategy): void {
            ownProps.service.addBot(ownProps.game.gameId, "classic", strategy);
        },
        joinGame(): void {
            ownProps.service.joinLobby(ownProps.game.gameId, "classic");
        },
        leaveGame(): void {
            ownProps.service.leaveLobby(ownProps.game.gameId);
        },
    }
}

class GameListItemInternal extends React.PureComponent<GameListItem.Props> {
    public render() {
        return <>
            <div className="game-list-item">
                <div className="game-name">Game {this.props.game.gameId}</div>
                <div className={classNames("players",
                    { "-full": this.props.game.players.length === 4 })}>{this.props.game.players.length}</div>
            </div>
            <div className="game-list-sub">
                {this.props.game.players.map(player => this.renderPlayer(player))}
                {this.renderButtons()}
            </div>
        </>;
    }

    private renderButtons() {
        if (this.props.game.players.length === 4) {
            return <div className="button-group game-controls">
                <a className="button" href={`/game/#${this.props.game.gameId}`} target="_blank">
                    Open
                </a>
            </div>
        }

        return (
            <div className="button-group game-controls">
                {this.props.game.players.some(p => p.userId === this.props.userId)
                    ? <div className="button" onClick={this.props.leaveGame}>
                        Leave
                    </div>
                    : <div className="button" onClick={this.props.joinGame}>
                        Join
                    </div>
                }
                <div className="button" onClick={this.addRandomBot}>
                    + Random
                </div>
                <div className="button" onClick={this.addDuckBot}>
                    + Duck
                </div>
                <div className="button" onClick={this.addGottaTryBot}>
                    + GT
                </div>
            </div>
        )
    }

    private renderPlayer(player: LobbyPlayer) {
        if (player.type === "bot") {
            return (
                <div key={player.userId} className="player -is-bot">
                    {player.strategy} {player.userId.substr(0, 8)}
                </div>
            )
        }
        return (
            <div key={player.userId} className="player">
                {this.props.users.userNamesByUserId[player.userId] !== undefined
                    ? this.props.users.userNamesByUserId[player.userId]
                    : "Loading"}
            </div>
        )
    }

    private addRandomBot = () => this.props.addBot("random");
    private addDuckBot = () => this.props.addBot("duck");
    private addGottaTryBot = () => this.props.addBot("gotta_try");
}

export const GameListItem = connect(mapStateToProps, mapDispatchToProps)(GameListItemInternal);
