import * as React from "react";
import { connect } from "react-redux";
import { GameAppState, User } from "../state/types";
import { Seat } from "../types";
import { POSITION_FOR_BOTTOM_SEAT } from "../util/seatPositions";

export namespace ScoreTable {
  export interface StoreProps {
    bottomSeat: Seat;
    top: User | undefined;
    right: User | undefined;
    bottom: User | undefined;
    left: User | undefined;
    scores: number[][];
  }

  export type Props = StoreProps;
}

class ScoreTableInternal extends React.Component<ScoreTable.Props> {
  public render() {
    if (this.props.bottomSeat === undefined) {
      return <div className="score-table"></div>;
    }
    return (
      <div className="score-table">
        <table>
          <thead>
            <tr>
              {POSITION_FOR_BOTTOM_SEAT[this.props.bottomSeat].map(position => (
                <th title={this.props[position]?.name ?? "loading..."}>{nameToInitials(this.props[position])}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {this.props.scores.map(scoreRow => (
              <tr>
                {scoreRow.map(score => (
                  <td>{score}</td>
                ))}
              </tr>
            ))}
            {this.props.scores.length > 0 ? (
              <tr>
                {totalMoneyWon(this.props.scores).map(money => (
                  <td className="totals">{money}</td>
                ))}
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    );
  }
}

function nameToInitials(user: User | undefined) {
  if (user === undefined) {
    return "...";
  }
  if (user.name.startsWith("Bot (")) {
    return user.userId.substr(0, 2);
  }
  return user.name
    .split(" ", 2)
    .map(s => s[0])
    .join("");
}

function totalPointsTaken(scores: number[][]): number[] {
  return scores.reduce((totalRow, currentRow) => totalRow.map((value, index) => value + currentRow[index]));
}

function totalMoneyWon(scores: number[][]): number[] {
  const totals = totalPointsTaken(scores);
  const sumOfTotals = totals.reduce((acc, value) => acc + value);
  return totals.map(playerTotal => sumOfTotals - 4 * playerTotal);
}

function mapStateToProps(state: GameAppState): ScoreTable.StoreProps {
  return {
    ...state.game
  };
}

export const ScoreTable = connect(mapStateToProps)(ScoreTableInternal);
