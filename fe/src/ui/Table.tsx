import * as React from "react";
import { Card } from "../types";
import * as classNames from "classnames";
import { Hand } from "./Hand";

export interface TableProps {
  north: Card[];
  east: Card[];
  south: Card[];
  west: Card[];
}

export class Table extends React.Component<TableProps, {}> {
  constructor(props?: TableProps, context?: any) {
    super(props, context);
  }

  render() {
    return (
      <div className={classNames("table")}>
        <div className="north">
          <Hand cards={this.props.north} playable={false} />
        </div>
        <div className="east">
          <Hand cards={this.props.east} playable={false} />
        </div>
        <div className="south">
          <Hand cards={this.props.south} playable={true} />
        </div>
        <div className="west">
          <Hand cards={this.props.west} playable={false} />
        </div>
      </div>
    );
  }
}
