import * as React from "react";
import { Seat } from "../types";
import { GameState, GameContext, ClaimStatus } from "../state/types";
import { POSITIONS } from "../util/seatPositions";

export namespace ClaimResponse {
  export interface Props {
    game: GameState;
    context: GameContext;
  }
}

export class ClaimResponse extends React.Component<ClaimResponse.Props> {
  public render() {
    if (this.props.game.bottomSeat === undefined) {
      return null;
    }
    const claimsAwaitingResponse = this.getClaimsAwaitingResponse();
    if (claimsAwaitingResponse.length === 0) {
      return null;
    }
    const positions = POSITIONS[this.props.game.bottomSeat];
    return (
      <div className="claim-response">
        {claimsAwaitingResponse.map(seat => [
          <div key={seat}>
            {this.props.game[positions[seat]]?.name ?? "loading..."} has claimed the rest of the tricks.
          </div>,
          <div key={seat + "accept"}>
            {!this.props.game.spectatorMode && <button onClick={this.handleAccept(seat)}>Accept</button>}
          </div>,
          <div key={seat + "reject"}>
            {!this.props.game.spectatorMode && <button onClick={this.handleReject(seat)}>Reject</button>}
          </div>
        ])}
      </div>
    );
  }

  private getClaimsAwaitingResponse(): Seat[] {
    const activePlayerSeat = this.props.game.spectatorMode ? undefined : this.props.game.bottomSeat;
    return Object.entries(this.props.game.claims)
      .filter(([seat, claimStatus]) => seat !== activePlayerSeat && claimStatus !== undefined)
      .filter(([_seat, claimStatus]) => activePlayerSeat !== undefined && claimStatus![activePlayerSeat] !== "ACCEPT")
      .filter(([_seat, claimStatus]) => Object.entries(claimStatus!).every(([_key, value]) => value !== "REJECT"))
      .map(([seat, _claimStatus]) => seat as Seat);
  }

  private handleAccept = (claimer: Seat) => () => {
    this.props.context.service.acceptClaim(claimer);
  };

  private handleReject = (claimer: Seat) => () => {
    this.props.context.service.rejectClaim(claimer);
  };
}
