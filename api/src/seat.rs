use crate::sql_json;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::{Debug, Display},
};

#[repr(u8)]
#[serde(rename_all = "snake_case")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Seat {
    North,
    East,
    South,
    West,
}

impl Seat {
    pub const VALUES: [Seat; 4] = [Seat::North, Seat::East, Seat::South, Seat::West];

    pub fn all<F>(f: F) -> bool
    where
        F: Fn(Seat) -> bool,
    {
        f(Seat::North) && f(Seat::East) && f(Seat::South) && f(Seat::West)
    }

    pub fn idx(&self) -> usize {
        *self as usize
    }

    pub fn left(&self) -> Self {
        match self {
            Seat::North => Seat::East,
            Seat::East => Seat::South,
            Seat::South => Seat::West,
            Seat::West => Seat::North,
        }
    }

    pub fn right(&self) -> Self {
        match self {
            Seat::North => Seat::West,
            Seat::East => Seat::North,
            Seat::South => Seat::East,
            Seat::West => Seat::South,
        }
    }

    pub fn across(&self) -> Self {
        match self {
            Seat::North => Seat::South,
            Seat::East => Seat::West,
            Seat::South => Seat::North,
            Seat::West => Seat::East,
        }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

sql_json!(Seat);
