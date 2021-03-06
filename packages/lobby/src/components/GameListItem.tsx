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

        startGame(): void;
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
        startGame(): void {
            ownProps.service.startGame(ownProps.game.gameId);
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
        if (this.props.game.startedAt !== undefined) {
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
                <select className="select button add-bot-select" value="__add_bot" onChange={this.addBot}>
                    <option disabled={true} value="__add_bot">Add bot</option>
                    <option value="random" className="button">Random</option>
                    <option value="duck" className="button">Duck</option>
                    <option value="gotta_try" className="button">Gotta Try</option>
                    <option value="heuristic" className="button">Heuristic</option>
                    <option value="simulate" className="button">Simulate</option>
                    <option value="neural_net" className="button">Neural Net</option>
                </select>
                {this.props.game.players.length >= 4 && (
                    <div className="button" onClick={this.props.startGame}>
                            Start!
                    </div>
                )}
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

    private addBot = (select: React.ChangeEvent<HTMLSelectElement>) => {
        this.props.addBot(select.target.value as BotStrategy);
    };
}

export const GameListItem = connect(mapStateToProps, mapDispatchToProps)(GameListItemInternal);
