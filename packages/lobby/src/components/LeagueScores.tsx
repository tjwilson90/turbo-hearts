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
                <h3>Leaderboard</h3>
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
        return (
            <div className="welcome-notes">
                <h3>Welcome</h3>
                <p className="announcement">
                    Feel free to play anytime, but look for regular games at <b>noon</b> and <b>8:30pm</b> Pacific.
                </p>
                <p>Note two differences to WotC style Turbo Hearts:</p>
                <ol>
                    <li>A "game" is always 4 hands.</li>
                    <li>
                        We've modified the Keeper. It's colloquially known as the "Keep It Interesting". It works like
                        so:
                        <ul>
                            <li>There's a round of charging.</li>
                            <li>If there's a charge, then play proceeds as if it was a Keeper.</li>
                            <li>If there isn't a charge, then all players pass three cards to the center.</li>
                            <li>Each player receives three random cards from the center.</li>
                            <li>There's another round of charging.</li>
                            <li>Play proceeds regardless of whether there is a charge.</li>
                        </ul>
                    </li>
                </ol>
                <p>
                    Also, if you add more than 4 players to a game and press Start, 4 players will be randomly chosen to
                    play.
                </p>
            </div>
        );
    }
}

export const LeagueScores = connect(mapStateToProps)(LeagueScoresInternal);
