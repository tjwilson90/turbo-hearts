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
    const activePlayerSeat = this.props.game.spectatorMode ? undefined : this.props.game.bottomSeat;
    const positions = POSITIONS[this.props.game.bottomSeat];
    return (
      <div className="claim-response">
        {claimsAwaitingResponse.map(([seat, claimStatus]) => {
          const currentUserMadeClaim = seat === activePlayerSeat;
          const currentUserHasResponded = activePlayerSeat !== undefined && claimStatus[activePlayerSeat] === "ACCEPT";
          return [
            <div key={seat}>
              {currentUserMadeClaim ? "You have claimed" : (this.props.game[positions[seat]]?.name ?? seat) + " has claimed"} the rest of the tricks.
              <br />
              {this.getAcceptCountText(claimStatus)}
            </div>,
            <div key={seat + "accept"}>
              {!this.props.game.spectatorMode && !currentUserHasResponded && (
                <button onClick={this.handleAccept(seat)}>Accept</button>
              )}
            </div>,
            <div key={seat + "reject"}>
              {!this.props.game.spectatorMode && !currentUserHasResponded && (
                <button onClick={this.handleReject(seat)}>Reject</button>
              )}
            </div>
          ];
        })}
      </div>
    );
  }

  private getAcceptCountText(claimStatus: ClaimStatus): string {
    const count = Object.entries(claimStatus).reduce((acc, entry) => (entry[1] ? acc + 1 : acc), 0) - 1; // -1: don't count the player who made the claim
    if (count == 0) {
      return "Waiting for responses...";
    }
    if (count == 1) {
      return "1 player has accepted.";
    }
    return `${count} players have accepted.`;
  }

  private getClaimsAwaitingResponse(): [Seat, ClaimStatus][] {
    return Object.entries(this.props.game.claims)
      .filter(([_seat, claimStatus]) => claimStatus !== undefined)
      .filter(([_seat, claimStatus]) => Object.entries(claimStatus!).every(([_key, value]) => value !== "REJECT"))
      .map(([seat, claimStatus]) => [seat as Seat, claimStatus!]);
  }

  private handleAccept = (claimer: Seat) => () => {
    this.props.context.service.acceptClaim(claimer);
  };

  private handleReject = (claimer: Seat) => () => {
    this.props.context.service.rejectClaim(claimer);
  };
}
