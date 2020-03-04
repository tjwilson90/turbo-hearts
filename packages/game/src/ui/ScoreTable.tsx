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
    scores: number[][]
  }

  export type Props = StoreProps;
}

class ScoreTableInternal extends React.Component<ScoreTable.Props> {
  public render() {
    if (this.props.bottomSeat === undefined) {
      return <div className="score-table"></div>
    }
    return <div className="score-table">
      <tr>{POSITION_FOR_BOTTOM_SEAT[this.props.bottomSeat].map(position => <th title={this.props[position]?.name ?? "loading..."}>{nameToInitials(this.props[position])}</th>)}</tr>
      {this.props.scores.map(scoreRow => <tr>{scoreRow.map(score => <td>{score}</td>)}</tr>)}
    </div>;
  }
}

function nameToInitials(user: User | undefined) {
  if (user === undefined) {
    return "...";
  }
  if (user.name.startsWith("Bot (")) {
    return user.userId.substr(0, 2);
  }
  return user.name.split(" ", 2).map(s => s[0]).join("");
}

function mapStateToProps(state: GameAppState): ScoreTable.StoreProps {
  return {
    ...state.game,
  };
}

export const ScoreTable = connect(mapStateToProps)(ScoreTableInternal);
