use std::collections::BTreeSet;
use std::ops::{Index, IndexMut};

/// A map from `bool` to `BTreeSet<N>`.
#[derive(Debug, Clone)]
pub struct BinaryValueMultimap<N>([BTreeSet<N>; 2]);

impl<N: Ord> Default for BinaryValueMultimap<N> {
    fn default() -> Self {
        BinaryValueMultimap([BTreeSet::default(), BTreeSet::default()])
    }
}

impl<N: Ord> Index<bool> for BinaryValueMultimap<N> {
    type Output = BTreeSet<N>;

    fn index(&self, index: bool) -> &Self::Output {
        &self.0[if index { 1 } else { 0 }]
    }
}

impl<N: Ord> IndexMut<bool> for BinaryValueMultimap<N> {
    fn index_mut(&mut self, index: bool) -> &mut Self::Output {
        &mut self.0[if index { 1 } else { 0 }]
    }
}
