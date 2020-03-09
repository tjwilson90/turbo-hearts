import * as React from "react";
import { connect } from "react-redux";
import { GameAppState, GameContext, User, GameState } from "../state/types";
import { Nameplate } from "./Nameplate";
import { TurboHeartsStage } from "../view/TurboHeartsStage";
import { UserDispatcher } from "../state/UserDispatcher";
import { ChatLog } from "./ChatLog";
import { PlayHistory } from "./PlayHistory";
import { ScoreTable } from "./ScoreTable";
import { ChatInput } from "./ChatInput";
import { Action, TurboHearts, emptyStateSnapshot } from "../game/stateSnapshot";
import { Card, Pass } from "../types";
import { emptyArray } from "../util/array";
import { ClaimResponse } from "./ClaimResponse";

const directionText: { [P in Pass]: string } = {
  left: "left",
  right: "right",
  across: "across",
  keeper: "in"
};

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

  export interface State {
    action: Action;
    picks: Card[];
    snapshot: TurboHearts.StateSnapshot;
  }
}

class GameAppInternal extends React.Component<GameApp.Props, GameApp.State> {
  private canvasRef = React.createRef<HTMLCanvasElement>();
  private stage: TurboHeartsStage = undefined!;

  public state: GameApp.State = {
    action: "none",
    snapshot: emptyStateSnapshot(""),
    picks: []
  };

  public render() {
    return (
      <React.Fragment>
        <div className="canvas-container">
          <canvas ref={this.canvasRef}></canvas>
          <Nameplate user={this.props.game.top} className="top" action={this.props.game.topAction} />
          <Nameplate user={this.props.game.right} className="right" action={this.props.game.rightAction} />
          <Nameplate user={this.props.game.bottom} className="bottom" action={this.props.game.bottomAction} />
          <Nameplate user={this.props.game.left} className="left" action={this.props.game.leftAction} />
          {this.state.action === "pass" && (
            <div className="input">
              <div>Choose 3 cards to pass {directionText[this.state.snapshot.pass]}</div>
              <button onClick={this.handlePass} disabled={this.state.picks.length !== 3}>
                Pass
              </button>
            </div>
          )}
          {this.state.action === "charge" && (
            <div className="input">
              <div>Charge 0 or more cards</div>
              <button onClick={this.handleCharge}>Charge</button>
            </div>
          )}
          <ClaimResponse game={this.props.game} context={this.props.context} />
          <div className="claim">
            <button onClick={this.handleClaim}>Claim</button>
          </div>
        </div>
        <div className="sidebar">
          <div className="game-data">
            <PlayHistory />
            <ScoreTable />
          </div>
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
    this.stage = new TurboHeartsStage(this.canvasRef.current, this.props.me.userId, this.props.context.service, () =>
      this.props.context.eventSource.connect()
    );
    this.props.context.snapshotter.on("snapshot", snap => {
      this.setState({ snapshot: snap.next });
      this.stage.acceptSnapshot(snap);
    });
    this.props.context.eventSource.once("end_replay", this.stage.endReplay);
    this.stage.on("pick", picks => {
      this.setState({ picks });
      if (picks.length === 1 && this.state.action === "play") {
        this.stage.setAction("none", emptyArray(), true);
        this.props.context.service.playCard(picks[0]);
      }
    });
    this.stage.on("action", action => {
      this.setState({ action });
    });
  }

  public componentDidUpdate(prevProps: GameApp.Props) {
    if (prevProps.game.spectatorMode !== this.props.game.spectatorMode && this.props.game.spectatorMode) {
      this.stage.enableSpectatorMode();
    }
  }

  private handlePass = () => {
    if (this.state.picks.length === 3) {
      this.stage.setAction("none", emptyArray(), true);
      this.props.context.service.passCards(this.state.picks);
    }
  };

  private handleCharge = () => {
    this.stage.setAction("none", emptyArray(), true);
    this.props.context.service.chargeCards(this.state.picks);
  };

  private handleChat = (message: string) => {
    this.props.context.service.chat(message);
  };

  private handleClaim = () => {
    this.props.context.service.claim();
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
