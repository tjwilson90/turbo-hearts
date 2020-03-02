import * as React from "react";
import { connect } from "react-redux";
import { GameAppState, GameContext, User, GameState } from "../state/types";
import { Nameplate } from "./Nameplate";
import { TurboHeartsStage } from "../view/TurboHeartsStage";
import { UserDispatcher } from "../state/UserDispatcher";
import { ChatLog } from "./ChatLog";
import { ChatInput } from "./ChatInput";

export namespace GameApp {
  export interface OwnProps {
    userDispatcher: UserDispatcher;
  }

  export interface StoreProps {
    me: User;
    context: GameContext;
    game: GameState;
  }

  export type Props = OwnProps & StoreProps;
}

class GameAppInternal extends React.Component<GameApp.Props> {
  private canvasRef = React.createRef<HTMLCanvasElement>();

  public render() {
    return (
      <React.Fragment>
        <div className="canvas-container">
          <canvas ref={this.canvasRef}></canvas>
          <Nameplate user={this.props.game.top} className="top" />
          <Nameplate user={this.props.game.right} className="right" />
          <Nameplate user={this.props.game.bottom} className="bottom" />
          <Nameplate user={this.props.game.left} className="left" />
        </div>
        <div className="sidebar">
          <ChatLog />
          <ChatInput onChat={this.handleChat} />
        </div>
      </React.Fragment>
    );
  }

  public componentDidMount() {
    if (this.canvasRef.current == null) {
      return;
    }
    const animator = new TurboHeartsStage(
      this.canvasRef.current,
      this.props.me.userId,
      this.props.context.service,
      () => this.props.context.eventSource.connect()
    );
    this.props.context.snapshotter.on("snapshot", animator.acceptSnapshot);
    this.props.context.eventSource.once("end_replay", animator.endReplay);
  }

  private handleChat = (message: string) => {
    this.props.context.service.chat(message);
  };
}

function mapStateToProps(state: GameAppState): GameApp.StoreProps {
  return {
    me: state.users.me,
    context: state.context,
    game: state.game
  };
}

export const GameApp = connect(mapStateToProps)(GameAppInternal);
