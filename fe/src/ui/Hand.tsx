import * as React from "react";
import { Card } from "../types";
import * as classNames from "classnames";

export interface HandProps {
  cards: Card[];
  playable: boolean;
}

export class Hand extends React.Component<HandProps, {}> {
  constructor(props?: HandProps, context?: any) {
    super(props, context);
  }

  render() {
    return (
      <div className={classNames("hand", { playable: this.props.playable })}>
        {this.props.cards.map((c, idx) => (
          <div key={idx} className="card">
            <img src={`/assets/cards/${c}.svg`} />
          </div>
        ))}
      </div>
    );
  }
}
