use core::{fmt, hash};

pub trait ValidatorIndex:
    Eq + Ord + Clone + Copy + Send + Sync + fmt::Debug + fmt::Display + hash::Hash + Into<usize>
{
}

impl<I> ValidatorIndex for I where
    I: Eq + Ord + Clone + Copy + Send + Sync + fmt::Debug + fmt::Display + hash::Hash + Into<usize>
{
}
