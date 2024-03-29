use crate::{Card, Cards, ChargingRules, PassDirection, Player, Seat, Seed, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameEvent {
    Ping,
    EndReplay {
        subscribers: HashSet<UserId>,
    },
    JoinGame {
        user_id: UserId,
    },
    LeaveGame {
        user_id: UserId,
    },
    Chat {
        user_id: UserId,
        message: String,
    },
    Sit {
        north: Player,
        east: Player,
        south: Player,
        west: Player,
        rules: ChargingRules,
        seed: Seed,
    },
    Deal {
        north: Cards,
        east: Cards,
        south: Cards,
        west: Cards,
        pass: PassDirection,
    },
    StartPassing,
    PassStatus {
        north_done: bool,
        east_done: bool,
        south_done: bool,
        west_done: bool,
    },
    SendPass {
        from: Seat,
        cards: Cards,
    },
    RecvPass {
        to: Seat,
        cards: Cards,
    },
    HiddenSendPass {
        from: Seat,
        count: usize,
    },
    HiddenRecvPass {
        to: Seat,
        count: usize,
    },
    StartCharging,
    ChargeStatus {
        next_charger: Option<Seat>,
        north_done: bool,
        east_done: bool,
        south_done: bool,
        west_done: bool,
    },
    BlindCharge {
        seat: Seat,
        count: usize,
    },
    Charge {
        seat: Seat,
        cards: Cards,
    },
    RevealCharges {
        north: Cards,
        east: Cards,
        south: Cards,
        west: Cards,
    },
    Play {
        seat: Seat,
        card: Card,
    },
    PlayStatus {
        next_player: Seat,
        legal_plays: Cards,
    },
    StartTrick {
        leader: Seat,
    },
    EndTrick {
        winner: Seat,
    },
    Claim {
        seat: Seat,
        hand: Cards,
    },
    AcceptClaim {
        claimer: Seat,
        acceptor: Seat,
    },
    RejectClaim {
        claimer: Seat,
        rejector: Seat,
    },
    HandComplete {
        north_score: i16,
        east_score: i16,
        south_score: i16,
        west_score: i16,
    },
    GameComplete {
        seed: Seed,
    },
}

impl GameEvent {
    pub fn is_ping(&self) -> bool {
        match self {
            GameEvent::Ping => true,
            _ => false,
        }
    }

    pub fn is_stable(&self) -> bool {
        use GameEvent::*;
        match self {
            Ping | EndReplay { .. } | JoinGame { .. } | LeaveGame { .. } => false,
            _ => true,
        }
    }

    pub fn redact(&self, seat: Option<Seat>, rules: ChargingRules) -> GameEvent {
        match self {
            GameEvent::Sit {
                north,
                east,
                south,
                west,
                rules,
                seed,
            } => GameEvent::Sit {
                north: *north,
                east: *east,
                south: *south,
                west: *west,
                rules: *rules,
                seed: seed.redact(),
            },
            GameEvent::Deal {
                north,
                east,
                south,
                west,
                pass,
            } => match seat {
                Some(Seat::North) => GameEvent::Deal {
                    north: *north,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Some(Seat::East) => GameEvent::Deal {
                    north: Cards::NONE,
                    east: *east,
                    south: Cards::NONE,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Some(Seat::South) => GameEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: *south,
                    west: Cards::NONE,
                    pass: *pass,
                },
                Some(Seat::West) => GameEvent::Deal {
                    north: Cards::NONE,
                    east: Cards::NONE,
                    south: Cards::NONE,
                    west: *west,
                    pass: *pass,
                },
                None => self.clone(),
            },
            GameEvent::SendPass { from, cards } => match seat {
                Some(seat) if seat != *from => GameEvent::HiddenSendPass {
                    from: *from,
                    count: cards.len(),
                },
                _ => self.clone(),
            },
            GameEvent::RecvPass { to, cards } => match seat {
                Some(seat) if seat != *to => GameEvent::HiddenRecvPass {
                    to: *to,
                    count: cards.len(),
                },
                _ => self.clone(),
            },
            GameEvent::Charge {
                seat: charger,
                cards,
            } => match seat {
                Some(seat) if seat != *charger && rules.blind() => GameEvent::BlindCharge {
                    seat: *charger,
                    count: cards.len(),
                },
                _ => self.clone(),
            },
            GameEvent::PlayStatus { next_player, .. } => match seat {
                Some(seat) if seat != *next_player => GameEvent::PlayStatus {
                    next_player: *next_player,
                    legal_plays: Cards::NONE,
                },
                _ => self.clone(),
            },
            _ => self.clone(),
        }
    }
}
