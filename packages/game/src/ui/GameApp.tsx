import * as React from "react";
import { connect } from "react-redux";
import { GameAppState, User } from "../state/types";

export namespace GameApp {
  export interface OwnProps {}
  export interface StoreProps {
    me: User;
  }
  export type Props = OwnProps & StoreProps;
}

class GameAppInternal extends React.Component<GameApp.Props> {
  public render() {
    return <div>User: {this.props.me.name}</div>;
  }
}

function mapStateToProps(state: GameAppState): GameApp.StoreProps {
  return {
    me: state.users.me
  };
}

export const GameApp = connect(mapStateToProps)(GameAppInternal);
