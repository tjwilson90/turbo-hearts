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

const SECRET_KEY_CODE = "`";

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
    isSecretButtonHeld: boolean;
  }
}

class GameAppInternal extends React.Component<GameApp.Props, GameApp.State> {
  private canvasRef = React.createRef<HTMLCanvasElement>();
  private stage: TurboHeartsStage = undefined!;

  public state: GameApp.State = {
    action: "none",
    snapshot: emptyStateSnapshot(""),
    picks: [],
    isSecretButtonHeld: false,
  };

  public render() {
    const passAny = this.state.isSecretButtonHeld && this.state.snapshot.pass === "keeper";
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
              <div>Choose {passAny ? "any number of" : "3"} cards to pass {directionText[this.state.snapshot.pass]}</div>
              <button onClick={this.handlePass} disabled={!passAny && this.state.picks.length !== 3}>
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
          {!this.props.game.spectatorMode && <div className="claim">
            <button disabled={!this.isClaimButtonEnabled()} onClick={this.handleClaim}>Claim</button>
          </div>}
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
    document.addEventListener("keydown", this.onKeydown);
    document.addEventListener("keyup", this.onKeyup);
  }

  public componentDidUpdate(prevProps: GameApp.Props) {
    if (prevProps.game.spectatorMode !== this.props.game.spectatorMode && this.props.game.spectatorMode) {
      this.stage.enableSpectatorMode();
    }
  }

  private isClaimButtonEnabled() {
    // Only allow claims during play, not during pass or charge.
    if ([this.props.game.topAction, this.props.game.rightAction, this.props.game.bottomAction, this.props.game.leftAction].some(action => action === "charge" || action === "pass")) {
      return false;
    }
    // Only allow claim if current player does not have an active claim (one that has not yet been rejected).
    const currentPlayerClaimStatus = this.props.game.claims[this.props.game.bottomSeat];
    if (currentPlayerClaimStatus === undefined) {
      return true;
    }
    return !Object.entries(currentPlayerClaimStatus!).every(([_key, value]) => value !== "REJECT");
  }

  private handlePass = () => {
    if (this.state.picks.length === 3 || (this.state.isSecretButtonHeld && this.state.snapshot.pass === "keeper")) {
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

  private onKeydown = (evt: KeyboardEvent) => {
    if (evt.key === SECRET_KEY_CODE) {
      this.setState({ isSecretButtonHeld: true });
    }
  };

  private onKeyup = (evt: KeyboardEvent) => {
    if (evt.key === SECRET_KEY_CODE) {
      this.setState({ isSecretButtonHeld: false });
    }
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
