import * as React from "react";
import { connect } from "react-redux";
import { TurboHearts } from "../game/stateSnapshot";
import { GameAppState } from "../state/types";
import { Seat } from "../types";

export namespace TrickLog {
  export interface StoreProps {
    tricks: TurboHearts.Trick[];
    bottomSeat: Seat;
  }

  export type Props = StoreProps;
}

const LEADER_OFFSET = {
  north: 0,
  east: 3,
  south: 2,
  west: 1
};

const BOTTOM_OFFSET = {
  north: 2,
  east: 3,
  south: 0,
  west: 1
};

class TrickLogInternal extends React.Component<TrickLog.Props> {
  public render() {
    if (this.props.tricks.length === 0) {
      return <div className="trick-log"></div>;
    }
    return <div className="trick-log">{this.renderNiceTrick(this.props.tricks[this.props.tricks.length - 1])}</div>;
  }

  private renderNiceTrick(trick: TurboHearts.Trick) {
    const leaderOffset = LEADER_OFFSET[trick.leader];
    const bottomOffset = BOTTOM_OFFSET[this.props.bottomSeat];
    if (trick.plays.length === 4) {
      return (
        <div className="trick-container">
          <div className="top">{trick.plays[(leaderOffset + bottomOffset) % 4]}</div>
          <div className="right">{trick.plays[(1 + leaderOffset + bottomOffset) % 4]}</div>
          <div className="bottom">{trick.plays[(2 + leaderOffset + bottomOffset) % 4]}</div>
          <div className="left">{trick.plays[(3 + leaderOffset + bottomOffset) % 4]}</div>
          <div className="center">LAST</div>
        </div>
      );
    } else {
      return (
        <div className="trick-container">
          <div className="top">
            {trick.plays[(leaderOffset + bottomOffset) % 4]}
            <br />
            {trick.plays[4 + ((leaderOffset + bottomOffset) % 4)]}
          </div>
          <div className="right">
            {trick.plays[(1 + leaderOffset + bottomOffset) % 4]}
            <br />
            {trick.plays[4 + ((1 + leaderOffset + bottomOffset) % 4)]}
          </div>
          <div className="bottom">
            {trick.plays[(2 + leaderOffset + bottomOffset) % 4]}
            <br />
            {trick.plays[4 + ((2 + leaderOffset + bottomOffset) % 4)]}
          </div>
          <div className="left">
            {trick.plays[(3 + leaderOffset + bottomOffset) % 4]}
            <br />
            {trick.plays[4 + ((3 + leaderOffset + bottomOffset) % 4)]}
          </div>
          <div className="center">LAST</div>
        </div>
      );
    }
  }
}

function mapStateToProps(state: GameAppState): TrickLog.StoreProps {
  return {
    tricks: state.game.tricks,
    bottomSeat: state.game.bottomSeat
  };
}

export const TrickLog = connect(mapStateToProps)(TrickLogInternal);
