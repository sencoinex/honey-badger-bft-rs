mod binary_value_multimap;
mod binary_values_multimap;
mod set;
pub use binary_value_multimap::*;
pub use binary_values_multimap::*;
pub use set::*;

use crate::Error;
use std::ops::Add;

const FALSE: u8 = 0b01;
const TRUE: u8 = 0b10;
const BOTH: u8 = 0b11;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryValues {
    False,
    True,
    Both,
}

impl BinaryValues {
    pub fn includes(&self, other: BinaryValues) -> bool {
        match self {
            Self::False => match other {
                Self::False => true,
                Self::True => false,
                Self::Both => false,
            },
            Self::True => match other {
                Self::False => false,
                Self::True => true,
                Self::Both => false,
            },
            Self::Both => true,
        }
    }

    pub fn single(&self) -> Option<bool> {
        match self {
            Self::False => Some(false),
            Self::True => Some(true),
            Self::Both => None,
        }
    }
}

impl From<bool> for BinaryValues {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

impl TryFrom<u8> for BinaryValues {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            FALSE => Ok(Self::False),
            TRUE => Ok(Self::True),
            BOTH => Ok(Self::Both),
            _ => Err(Error::InvalidBinaryValues { value }),
        }
    }
}

impl Into<u8> for BinaryValues {
    fn into(self) -> u8 {
        match self {
            Self::False => FALSE,
            Self::True => TRUE,
            Self::Both => BOTH,
        }
    }
}

impl Add for BinaryValues {
    type Output = BinaryValues;
    fn add(self, rhs: Self) -> Self {
        match self {
            Self::False => match rhs {
                Self::False => Self::False,
                Self::True => Self::Both,
                Self::Both => Self::Both,
            },
            Self::True => match rhs {
                Self::False => Self::Both,
                Self::True => Self::True,
                Self::Both => Self::Both,
            },
            Self::Both => Self::Both,
        }
    }
}
