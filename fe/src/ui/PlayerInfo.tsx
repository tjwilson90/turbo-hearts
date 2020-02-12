import * as React from "react";
import { Card, WithCharged, Player } from "../types";
import * as classNames from "classnames";
import { isCharged } from "../util/charges";

export interface PlayerInfoProps {
  player: Player;
}

function card(c: Card) {
  return <img src={`/assets/cards/${c}.svg`} />;
}

export class PlayerInfo extends React.Component<PlayerInfoProps, {}> {
  constructor(props?: PlayerInfoProps, context?: any) {
    super(props, context);
  }

  render() {
    return (
      <div className={classNames("player")}>
        <div>{this.props.player.name}</div>
        <div className="charges">
          {this.props.player.chargedTc && card("TC")}
          {this.props.player.chargedJd && card("JD")}
          {this.props.player.chargedAh && card("AH")}
          {this.props.player.chargedQs && card("QS")}
        </div>
      </div>
    );
  }
}
