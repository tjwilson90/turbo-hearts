import * as React from "react";
import { connect } from "react-redux";
import { GameAppState, GameContext, User, GameState } from "../state/types";
import { Nameplate } from "./Nameplate";
import { TurboHeartsStage } from "../view/TurboHeartsStage";
import { Dispatch } from "redoodle";
import { UserDispatcher } from "../state/UserDispatcher";

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
          <div id="chat-log" className="chat-log"></div>
          <textarea id="chat-input" className="chat-input" placeholder="Enter chat message..."></textarea>
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
    // const chatLog = document.getElementById("chat-log")!;
    // const chatAppender = async (message: ChatEvent) => {
    //   // TODO: fix race
    //   console.log(message);
    //   const users = await store.getState().context.service.getUsers([message.userId]);
    //   const div = document.createElement("div");
    //   div.classList.add("chat-message-container");
    //   const nameSpan = document.createElement("span");
    //   nameSpan.classList.add("chat-user");
    //   nameSpan.textContent = users[message.userId];
    //   div.appendChild(nameSpan);
    //   const messageSpan = document.createElement("span");
    //   messageSpan.classList.add("chat-message");
    //   messageSpan.textContent = message.message;
    //   div.appendChild(messageSpan);
    //   chatLog.appendChild(div);
    //   div.scrollIntoView();
    // };

    // const eventSource = new TurboHeartsEventSource(gameId);
    // eventSource.on("chat", chatAppender);

    // const snapshotter = new Snapshotter(userId);

    // snapshotter.on("snapshot", e => console.log(e));

    // function start() {
    //   eventSource.connect();
    // }

    // const animator = new TurboHeartsStage(
    //   document.getElementById("turbo-hearts") as HTMLCanvasElement,
    //   userId,
    //   store.getState().context.service,
    //   start
    // );
    // snapshotter.on("snapshot", animator.acceptSnapshot);
    // eventSource.once("end_replay", animator.endReplay);
    // new ChatInput(document.getElementById("chat-input") as HTMLTextAreaElement, store.getState().context.service);
  }
}

function mapStateToProps(state: GameAppState): GameApp.StoreProps {
  return {
    me: state.users.me,
    context: state.context,
    game: state.game
  };
}

export const GameApp = connect(mapStateToProps)(GameAppInternal);
