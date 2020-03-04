import * as React from "react";
import { connect } from "react-redux";
import { GameAppState } from "../state/types";

export namespace ScoreTable {
  export interface StoreProps {
    scores: number[][]
  }

  export type Props = StoreProps;
}

class ScoreTableInternal extends React.Component<ScoreTable.Props> {
  public render() {
    if (this.props.scores.length === 0) {
      return <div className="score-table"></div>;
    }
    return <div className="score-table">
      <tr><th>N</th><th>E</th><th>S</th><th>W</th></tr>
      {this.props.scores.map(scoreRow => <tr>{scoreRow.map(score => <td>{score}</td>)}</tr>)}
    </div>;
  }
}

function mapStateToProps(state: GameAppState): ScoreTable.StoreProps {
  return {
    scores: state.game.scores
  };
}

export const ScoreTable = connect(mapStateToProps)(ScoreTableInternal);
