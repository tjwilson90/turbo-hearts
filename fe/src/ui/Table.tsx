import * as React from "react";
import { Card, Player } from "../types";
import classNames from "classnames";
import { Hand } from "./Hand";
import { PlayerInfo } from "./PlayerInfo";

export interface TableProps {
  top: Player;
  right: Player;
  bottom: Player;
  left: Player;
  topPlays: Card[];
  rightPlays: Card[];
  bottomPlays: Card[];
  leftPlays: Card[];
}

export class Table extends React.Component<TableProps, {}> {
  constructor(props?: TableProps, context?: any) {
    super(props, context);
  }

  render() {
    return (
      <div className={classNames("table")}>
        <div className="top-info">
          <PlayerInfo player={this.props.top} />
        </div>
        <div className="top">
          <Hand
            charges={this.props.top}
            cards={this.props.top.hand}
            playable={false}
          />
        </div>
        <div className="right-info">
          <PlayerInfo player={this.props.right} />
        </div>
        <div className="right">
          <Hand
            charges={this.props.right}
            cards={this.props.right.hand}
            playable={false}
          />
        </div>
        <div className="bottom-info">
          <PlayerInfo player={this.props.bottom} />
        </div>
        <div className="bottom">
          <Hand
            charges={this.props.bottom}
            cards={this.props.bottom.hand}
            playable={true}
          />
        </div>
        <div className="left-info">
          <PlayerInfo player={this.props.left} />
        </div>
        <div className="left">
          <Hand
            charges={this.props.left}
            cards={this.props.left.hand}
            playable={false}
          />
        </div>
        <div className="center"></div>
        <div className="top-plays">
          <Hand cards={this.props.topPlays} playable={false} />
        </div>
        <div className="right-plays">
          <Hand cards={this.props.rightPlays} playable={false} />
        </div>
        <div className="bottom-plays">
          <Hand cards={this.props.bottomPlays} playable={false} />
        </div>
        <div className="left-plays">
          <Hand cards={this.props.leftPlays} playable={false} />
        </div>
      </div>
    );
  }
}
