import * as React from "react";
import { connect } from "react-redux";
import { LobbyState, UsersState } from "../state/types";
import { GameResult } from "../types";
import { calculateScores, GameScores, calculateLeaderboard, LeaderboardEntry } from "../utils/gameResults";
import { NiceCard as Card, NiceSuit as Suit } from "./Card";

type Sort = "user" | "points" | "games" | "ppg";

export namespace LeagueScores {
    export interface OwnProps {}

    export interface StoreProps {
        games: GameResult[];
        users: UsersState;
    }

    export interface DispatchProps {}

    export type Props = OwnProps & StoreProps & DispatchProps;

    export interface State {
        sort: Sort;
        reverse: boolean;
    }
}

function mapStateToProps(state: LobbyState): LeagueScores.StoreProps {
    return {
        games: state.leagues.games,
        users: state.users,
    };
}

class LeagueScoresInternal extends React.PureComponent<LeagueScores.Props, LeagueScores.State> {
    public state: LeagueScores.State = {
        sort: "points",
        reverse: false,
    };

    public render() {
        const allScores = this.props.games.map(calculateScores);
        return (
            <div className="league-scores-wrapper">
                <div className="leaderboard">{this.renderLeaderboard(allScores)}</div>
                <div className="welcome-notes">{this.renderNotes()}</div>
            </div>
        );
    }

    private renderLeaderboard(allScores: GameScores[]) {
        const leaderboard = calculateLeaderboard(allScores);
        const reverse = this.state.reverse ? -1 : 1;
        leaderboard.sort((a, b) => this.getSortFunction()(a, b) * reverse);
        return (
            <div>
                <h3>Leaderboard</h3>
                <table>
                    <thead>
                        <tr>
                            <th onClick={this.handleSortRequest("user")}>User</th>
                            <th onClick={this.handleSortRequest("points")}>Points</th>
                            <th onClick={this.handleSortRequest("games")}>Games</th>
                            <th onClick={this.handleSortRequest("ppg")}>PPG</th>
                        </tr>
                    </thead>
                    <tbody>
                        {leaderboard.map((entry) => (
                            <tr key={entry.userId}>
                                <td>{this.renderUserName(entry.userId)}</td>
                                <td>{entry.points}</td>
                                <td>{entry.games}</td>
                                <td>{Math.round(entry.points / entry.games)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        );
    }

    private handleSortRequest = (sort: Sort) => () => {
        if (this.state.sort === sort) {
            this.setState({
                sort: this.state.sort,
                reverse: !this.state.reverse,
            });
        } else {
            this.setState({
                sort,
                reverse: false,
            });
        }
    };

    private getSortFunction(): (a: LeaderboardEntry, b: LeaderboardEntry) => number {
        switch (this.state.sort) {
            case "points":
                return (a, b) => b.points - a.points;
            case "games":
                return (a, b) => b.games - a.games;
            case "ppg":
                return (a, b) => b.points / b.games - a.points / a.games;
            case "user":
                return (a, b) => {
                    const userA = this.renderUserName(a.userId);
                    const userB = this.renderUserName(b.userId);
                    if (userA == null || userB == null) {
                        return 0;
                    }
                    return userA.localeCompare(userB);
                };
        }
    }

    private renderUserName(userId: string | undefined) {
        if (userId === undefined) {
            return null;
        }
        return this.props.users.userNamesByUserId[userId] !== undefined
            ? this.props.users.userNamesByUserId[userId]
            : "Loading";
    }

    private renderNotes() {
        return (
            <React.Fragment>
                <h3>Rules</h3>
                <p>
                    A game of Turbo Hearts consists of four hands. Each hand proceeds roughly as follows with exceptions
                    and details listed afterwards.
                </p>
                <ol>
                    <li>Each player is dealt thirteen cards.</li>
                    <li>
                        Each player chooses three of their cards to pass away face down, then receives three cards back.
                    </li>
                    <li>
                        Each player chooses whether to "charge" certain cards. Charging a card reveals it to your
                        opponents, changes the rules on when it can be played, and changes how scores will be calculated
                        for the hand. Only four specific cards may be charged, the <Card card="QS" />,{" "}
                        <Card card="AH" />, <Card card="JD" />, and <Card card="TC" />.
                    </li>
                    <li>
                        Players take turns playing cards from their hands in batches of four or eight cards called
                        "tricks".
                        <ul>
                            <li>
                                The first play is always the <Card card="2C" /> made by the player holding the{" "}
                                <Card card="2C" />.
                            </li>
                            <li>Play within a trick proceeds clockwise.</li>
                            <li>
                                A trick is complete after four cards are played (with each player playing once) if there
                                are no cards remaining, or if the trick does not contain the nine with the same suit as
                                the first card played in the trick. Otherwise a trick is complete after eight cards are
                                played (with each player playing twice).
                            </li>
                            <li>
                                When a trick is complete, the player who played the highest card with the same suit as
                                the first card played in the trick wins all of the cards played in the trick and gets to
                                lead the next trick (play the first card).
                            </li>
                        </ul>
                    </li>
                    <li>
                        After all cards have been played, the cards that each player won are used to determine their
                        score for the hand.
                    </li>
                </ol>
                <h4>Passing</h4>
                <ol>
                    <li>
                        In the first hand each player passes cards to the player on their left and receives cards from
                        the player to their right.
                    </li>
                    <li>
                        In the second hand passing is reversed; each player passes to their right and receives from
                        their left.
                    </li>
                    <li>In the third hand players pass cards to and receive cards from the player across from them.</li>
                    <li>
                        In the fourth hand players initially do not pass; a charging phase happens immediately after the
                        deal.
                        <ul>
                            <li>
                                If one or more cards are charged then no passes are made and play proceeds after the
                                charging phase completes.
                            </li>
                            <li>
                                Otherwise, each player passes three cards to a shared pile, then randomly receives three
                                of the twelve passed cards. After this an additional charging phase happens.
                            </li>
                        </ul>
                    </li>
                </ol>
                <h4>Charging</h4>
                <p>
                    Charging proceeds according to the following process. Charging does not happen in turns, players may
                    charge in any order.
                </p>
                <ol>
                    <li>Initially all players must choose to charge zero or more cards.</li>
                    <li>
                        After a player charges zero or more cards they are done charging. They cannot change their
                        decision unless some other player charges a card.
                    </li>
                    <li>
                        After a player charges one or more cards each other player must choose to charge zero or more
                        additional cards.
                    </li>
                    <li>
                        Charging ends once all players are done charging, i.e., all four players charged no cards or
                        three players charged no cards after the other player charged one or more cards.
                    </li>
                </ol>
                <h4>Playing</h4>
                <p>
                    Players are restricted in which cards they are allowed to play by the following rules. Earlier rules
                    take precedence over later rules. If a rule would leave a player with no cards to play then the rule
                    does not apply.
                </p>
                <ol>
                    <li>
                        A <Suit suit="H" /> cannot be led if no player has won a <Suit suit="H" />.
                    </li>
                    <li>Each card in a trick must have the same suit as the first card in its trick.</li>
                    <li>
                        A charged <Card card="AH" /> cannot be played in a <Suit suit="H" /> trick unless a{" "}
                        <Suit suit="H" /> has been led in an earlier trick.
                    </li>
                    <li>
                        A charged <Card card="QS" /> cannot be played in a <Suit suit="S" /> trick unless a{" "}
                        <Suit suit="S" /> has been led in an earlier trick.
                    </li>
                    <li>
                        A charged <Card card="JD" /> cannot be played in a <Suit suit="D" /> trick unless a{" "}
                        <Suit suit="D" /> has been led in an earlier trick.
                    </li>
                    <li>
                        A charged <Card card="TC" /> cannot be played in a <Suit suit="C" /> trick unless a{" "}
                        <Suit suit="C" /> has been led in an earlier trick.
                    </li>
                </ol>
                <h4>Claiming</h4>
                <p>
                    At any point after play begins any player may claim the remaining tricks. This reveals that player's
                    hand to all players and gives each other player an opportunity to accept or reject the claim.
                </p>
                <p>
                    If all other players accept a claim then the player who claimed wins all remaining cards and the
                    hand ends. Otherwise play continues as before.
                </p>
                <h4>Scoring</h4>
                <p>Each player's score is determined after each hand via the following algorithm.</p>
                <ol>
                    <li>
                        If the <Card card="AH" /> was charged each player receives two points for each <Suit suit="H" />{" "}
                        they won, otherwise each player receives one point for each <Suit suit="H" /> they won.
                    </li>
                    <li>
                        If the <Card card="QS" /> was charged, the player who won the <Card card="QS" /> receives
                        twenty-six points, otherwise they receive thirteen points.
                    </li>
                    <li>
                        If a player won all thirteen <Suit suit="H" />s and the <Card card="QS" />, their score is
                        negated.
                    </li>
                    <li>
                        If the <Card card="JD" /> was charged, the player who won the <Card card="JD" /> receives
                        negative twenty points, otherwise they receive negative ten points.
                    </li>
                    <li>
                        If the <Card card="TC" /> was charged, the player who won the <Card card="TC" /> has their score
                        quadrupled, otherwise they have their score doubled.
                    </li>
                </ol>
                <p>
                    Each player's winnings are equal to the total score of all players minus four times their own score.
                    Positive scores are bad. Positive winnings are good.
                </p>
            </React.Fragment>
        );
    }
}

export const LeagueScores = connect(mapStateToProps)(LeagueScoresInternal);
