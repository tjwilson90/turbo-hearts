import { LobbyGame } from "../lobbySnapshotter";
import * as React from "react";
import { TurboHeartsLobbyService } from "../TurboHeartsLobbyService";
import classNames from "classnames";

interface IProps {
    games: {[gameId: string]: LobbyGame}
    subscriberUserIds: string[];
    messages: {
        message: string;
        userId: string;
        generated: boolean;
        date: Date;
    }[];
    service: TurboHeartsLobbyService;
}

interface IState {
    usernameById: {[userId: string]: string};
}

export class Lobby extends React.PureComponent<IProps, IState> {
    public state: IState = {
        usernameById: {},
    };

    public render() {
        return <div className="lobby-wrapper">
            <div className="game-list">
                <div className="game-list-item -selected">Lobby</div>
                {this.renderGamesList()}
            </div>
            <div className="message-list">
            </div>
            <div className="user-list">
            </div>
        </div>
    }

    private renderGamesList() {
        const sortedGames = Object.values(this.props.games);
        sortedGames.sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime());
        return sortedGames.map(game => (
            <>
                <div className="game-list-item">
                    <div className="carat"/>
                    <div className="game-name">Game {game.gameId}</div><div className={classNames("players", {"-full": game.players.length === 4})}>{game.players.length}</div>
                </div>
                <div className="game-list-sub">
                    {game.players.map(player => (
                        <div className={classNames("player", {"-is-bot": player.type === "bot"})}>
                            {player.userId}
                        </div>
                    ))}
                    <div className="button-group">
                        <div className="button">
                            Join
                        </div>
                        <div className="button">
                            Add Bot
                        </div>
                    </div>
                </div>
                </>
        ))
    }
}
