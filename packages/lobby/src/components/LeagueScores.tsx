import * as React from "react";
import { connect } from "react-redux";
import { LobbyState, UsersState } from "../state/types";
import { GameResult } from "../types";
import { calculateScores, GameScores, calculateLeaderboard } from "../utils/gameResults";

export namespace LeagueScores {
    export interface OwnProps {}

    export interface StoreProps {
        games: GameResult[];
        users: UsersState;
    }

    export interface DispatchProps {}

    export type Props = OwnProps & StoreProps & DispatchProps;
}

function mapStateToProps(state: LobbyState): LeagueScores.StoreProps {
    return {
        games: state.leagues.games,
        users: state.users
    };
}

class LeagueScoresInternal extends React.PureComponent<LeagueScores.Props> {
    public render() {
        const allScores = this.props.games.map(calculateScores);
        return (
            <div className="league-scores-wrapper">
                <div className="leaderboard">{this.renderLeaderboard(allScores)}</div>
                <div className="games">{this.renderGames(allScores)}</div>
            </div>
        );
    }

    private renderLeaderboard(allScores: GameScores[]) {
        const leaderboard = calculateLeaderboard(allScores);
        leaderboard.sort((a, b) => b.points - a.points);
        return (
            <div>
                <table>
                    <thead>
                        <tr>
                            <th>User</th>
                            <th>Points</th>
                            <th>Games</th>
                        </tr>
                    </thead>
                    <tbody>
                        {leaderboard.map(entry => (
                            <tr key={entry.userId}>
                                <td>{this.renderUserName(entry.userId)}</td>
                                <td>{entry.points}</td>
                                <td>{entry.games}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );
    }

    private renderUserName(userId: string | undefined) {
        if (userId === undefined) {
            return null;
        }
        return this.props.users.userNamesByUserId[userId] !== undefined
            ? this.props.users.userNamesByUserId[userId]
            : "Loading";
    }

    private renderGames(_allScores: GameScores[]) {
        return <div>Future recent game list</div>;
    }
}

export const LeagueScores = connect(mapStateToProps)(LeagueScoresInternal);
