use core::{fmt, hash};

pub trait ValidatorIndex:
    Eq
    + Ord
    + Clone
    + Copy
    + Send
    + Sync
    + fmt::Debug
    + fmt::Display
    + hash::Hash
    + Into<usize>
    + Into<u64>
    + AsRef<u64>
{
}

impl<I> ValidatorIndex for I where
    I: Eq
        + Ord
        + Clone
        + Copy
        + Send
        + Sync
        + fmt::Debug
        + fmt::Display
        + hash::Hash
        + Into<usize>
        + Into<u64>
        + AsRef<u64>
{
}
