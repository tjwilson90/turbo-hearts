use crate::{ChargingRules, PassDirection, Seat};
use std::mem;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GamePhase {
    PassLeft,
    ChargeLeft,
    PlayLeft,
    PassRight,
    ChargeRight,
    PlayRight,
    PassAcross,
    ChargeAcross,
    PlayAcross,
    ChargeKeeper1,
    PassKeeper,
    ChargeKeeper2,
    PlayKeeper,
    Complete,
}

impl GamePhase {
    pub fn next(&self, charged: bool) -> Self {
        assert_ne!(*self, GamePhase::Complete);
        if *self == GamePhase::ChargeKeeper1 && charged {
            GamePhase::PlayKeeper
        } else {
            unsafe { mem::transmute(*self as u8 + 1) }
        }
    }

    pub fn is_complete(&self) -> bool {
        *self == GamePhase::Complete
    }

    pub fn is_passing(&self) -> bool {
        use GamePhase::*;
        match self {
            PassLeft | PassRight | PassAcross | PassKeeper => true,
            _ => false,
        }
    }

    pub fn is_charging(&self) -> bool {
        use GamePhase::*;
        match self {
            ChargeLeft | ChargeRight | ChargeAcross | ChargeKeeper1 | ChargeKeeper2 => true,
            _ => false,
        }
    }

    pub fn is_playing(&self) -> bool {
        use GamePhase::*;
        match self {
            PlayLeft | PlayRight | PlayAcross | PlayKeeper => true,
            _ => false,
        }
    }

    pub fn first_charger(&self, rules: ChargingRules) -> Option<Seat> {
        if rules.free() {
            return None;
        }
        use GamePhase::*;
        Some(match self {
            PassLeft | ChargeLeft | PlayLeft => Seat::North,
            PassRight | ChargeRight | PlayRight => Seat::East,
            PassAcross | ChargeAcross | PlayAcross => Seat::South,
            _ => Seat::West,
        })
    }

    pub fn pass_receiver(&self, seat: Seat) -> Seat {
        use GamePhase::*;
        match self {
            PassLeft | ChargeLeft | PlayLeft => seat.left(),
            PassRight | ChargeRight | PlayRight => seat.right(),
            PassAcross | ChargeAcross | PlayAcross => seat.across(),
            _ => seat,
        }
    }

    pub fn pass_sender(&self, seat: Seat) -> Seat {
        use GamePhase::*;
        match self {
            PassLeft | ChargeLeft | PlayLeft => seat.right(),
            PassRight | ChargeRight | PlayRight => seat.left(),
            PassAcross | ChargeAcross | PlayAcross => seat.across(),
            _ => seat,
        }
    }

    pub fn direction(&self) -> PassDirection {
        use GamePhase::*;
        match self {
            PassLeft | ChargeLeft | PlayLeft => PassDirection::Left,
            PassRight | ChargeRight | PlayRight => PassDirection::Right,
            PassAcross | ChargeAcross | PlayAcross => PassDirection::Across,
            _ => PassDirection::Keeper,
        }
    }
}
