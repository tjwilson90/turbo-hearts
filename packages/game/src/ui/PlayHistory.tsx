import * as React from "react";
import { connect } from "react-redux";
import { TurboHearts } from "../game/stateSnapshot";
import { GameAppState } from "../state/types";
import { Seat, Card } from "../types";
import classNames from "classnames";

export namespace TrickLog {
  export interface StoreProps {
    tricks: TurboHearts.Trick[];
    localPass: TurboHearts.LocalPass | undefined;
    bottomSeat: Seat;
  }

  export type Props = StoreProps;
}

const LEADER_OFFSET = {
  north: 0,
  east: 3,
  south: 2,
  west: 1
};

const BOTTOM_OFFSET = {
  north: 2,
  east: 3,
  south: 0,
  west: 1
};

export type Suit = "S" | "H" | "D" | "C";
export const suitMap: { [key in Suit]: string } = {
  S: "♠",
  H: "♥",
  D: "♦",
  C: "♣"
};

export const colorMap: { [key in Suit]: string } = {
  S: "#000000",
  H: "#ff0000",
  D: "#0d00ff",
  C: "#008000"
};

function NiceCard(props: { card: Card }) {
  if (props.card == null) {
    return null;
  }
  let rank = props.card.substring(0, 1);
  if (rank === "T") {
    rank = "10";
  }
  const suit = props.card.substring(1) as Suit;

  return <span style={{ color: colorMap[suit] }}>{`${rank}${suitMap[suit]}`}</span>;
}

class PlayHistoryInternal extends React.Component<TrickLog.Props> {
  public render() {
    if (this.props.localPass !== undefined) {
      return (
        <div className="play-history">
          {this.props.localPass.sent && (
            <div className="pass-section">
              <div>Passed</div>
              <div className="pass-cards" style={{fontSize: this.passCardsSize(this.props.localPass.sent.length) + 'px'}}>
                {this.props.localPass.sent.map(card => (
                  <NiceCard card={card} />
                ))}
              </div>
            </div>
          )}
          {this.props.localPass.received && (
            <div className="pass-section">
              <div>Received</div>
              <div className="pass-cards" style={{fontSize: this.passCardsSize(this.props.localPass.received.length) + 'px'}}>
                {this.props.localPass.received.map(card => (
                  <NiceCard card={card} />
                ))}
              </div>
            </div>
          )}
        </div>
      );
    }
    if (this.props.tricks.length === 0 || this.props.bottomSeat === undefined) {
      return <div className="play-history"></div>;
    }
    return <div className="play-history">{this.renderNiceTrick(this.props.tricks[this.props.tricks.length - 1])}</div>;
  }

  private passCardsSize(passSize: number) {
    if (passSize <= 4) {
      return 15;
    }
    if (passSize <= 6) {
      return 14;
    }
    if (passSize <= 8) {
      return 13;
    }
    if (passSize <= 10) {
      return 12;
    }
    return 11;
  }

  private renderNiceTrick(trick: TurboHearts.Trick) {
    const leaderOffset = LEADER_OFFSET[trick.leader];
    const bottomOffset = BOTTOM_OFFSET[this.props.bottomSeat];
    const isLeader = (i: number) => (i + leaderOffset + bottomOffset) % 4 === 0;
    const card = (i: number, ofs = 0) => {
      return <NiceCard card={trick.plays[ofs + ((i + leaderOffset + bottomOffset) % 4)]} />;
    };
    if (trick.plays.length === 4) {
      return (
        <div className="trick-container">
          <div className={classNames("top", { leader: isLeader(0) })}>{card(0)}</div>
          <div className={classNames("right", { leader: isLeader(1) })}>{card(1)}</div>
          <div className={classNames("bottom", { leader: isLeader(2) })}>{card(2)}</div>
          <div className={classNames("left", { leader: isLeader(3) })}>{card(3)}</div>
          <div className={classNames("center")}>LAST</div>
        </div>
      );
    } else {
      return (
        <div className="trick-container">
          <div className={classNames("top", { leader: isLeader(0) })}>
            {card(0)}
            <br />
            {card(0, 4)}
          </div>
          <div className={classNames("right", { leader: isLeader(1) })}>
            {card(1)}
            <br />
            {card(1, 4)}
          </div>
          <div className={classNames("bottom", { leader: isLeader(2) })}>
            {card(2)}
            <br />
            {card(2, 4)}
          </div>
          <div className={classNames("left", { leader: isLeader(3) })}>
            {card(3)}
            <br />
            {card(3, 4)}
          </div>
          <div className="center">LAST</div>
        </div>
      );
    }
  }
}

function mapStateToProps(state: GameAppState): TrickLog.StoreProps {
  return {
    tricks: state.game.tricks,
    localPass: state.game.localPass,
    bottomSeat: state.game.bottomSeat
  };
}

export const PlayHistory = connect(mapStateToProps)(PlayHistoryInternal);
