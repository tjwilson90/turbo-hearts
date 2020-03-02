import * as React from "react";
import { User } from "../state/types";
import { Action } from "../game/stateSnapshot";

export namespace Nameplate {
  export interface Props {
    user: User | undefined;
    className: string;
    action: Action;
  }
}

export class Nameplate extends React.Component<Nameplate.Props> {
  public render() {
    return (
      <div className={"nameplate " + this.props.className + " " + (this.props.action !== "none" ? "to-play" : "")}>
        <span className="name">{this.props.user?.name ?? "loading..."}</span>
      </div>
    );
  }
}
