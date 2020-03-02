import * as React from "react";
import { User } from "../state/types";

export namespace Nameplate {
  export interface Props {
    user: User | undefined;
    className: string;
  }
}

export class Nameplate extends React.Component<Nameplate.Props> {
  public render() {
    return (
      <div className={"nameplate " + this.props.className}>
        <span className="name">{this.props.user?.name ?? "loading..."}</span>
      </div>
    );
  }
}
