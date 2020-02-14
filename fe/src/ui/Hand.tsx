import * as React from "react";
import { Card, WithCharged } from "../types";
import classNames from "classnames";
import { isCharged } from "../util/charges";

export interface HandProps {
  cards: Card[];
  playable: boolean;
  charges?: WithCharged;
}

export class Hand extends React.Component<HandProps, {}> {
  constructor(props?: HandProps, context?: any) {
    super(props, context);
  }

  render() {
    return (
      <div className={classNames("hand", { playable: this.props.playable })}>
        {this.props.cards.map((c, idx) => {
          const charged =
            this.props.charges !== undefined &&
            isCharged(c, this.props.charges);
          return (
            <div key={idx} className={classNames("card", { charged })}>
              <img src={`/assets/cards/${c}.svg`} />
            </div>
          );
        })}
      </div>
    );
  }
}
