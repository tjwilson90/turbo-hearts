use serde::{Deserialize, Serialize};
use std::{
    fmt,
    fmt::{Debug, Display},
    mem,
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

    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn left(self) -> Self {
        let index = (self as u8 + 1) % 4;
        unsafe { mem::transmute(index) }
    }

    pub fn across(self) -> Self {
        let index = (self as u8 + 2) % 4;
        unsafe { mem::transmute(index) }
    }

    pub fn right(self) -> Self {
        let index = (self as u8 + 3) % 4;
        unsafe { mem::transmute(index) }
    }
}

impl Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

sql_json!(Seat);
