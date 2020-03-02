import * as React from "react";

export namespace Nameplate {
  export interface Props {
    name: string;
    className: string;
  }
}

export class Nameplate extends React.Component<Nameplate.Props> {
  public render() {
    return (
      <div className={"nameplate " + this.props.className}>
        <span className="name">{this.props.name}</span>
      </div>
    );
  }
}
