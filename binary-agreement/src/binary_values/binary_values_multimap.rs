use super::BinaryValues;
use std::collections::BTreeSet;
use std::ops::{Index, IndexMut};

/// A map from `BinaryValues` to `BTreeSet<N>`.
#[derive(Debug, Clone)]
pub struct BinaryValuesMultimap<N>([BTreeSet<N>; 3]);

impl<N: Ord> Default for BinaryValuesMultimap<N> {
    fn default() -> Self {
        BinaryValuesMultimap([
            BTreeSet::default(),
            BTreeSet::default(),
            BTreeSet::default(),
        ])
    }
}

impl<N: Ord> Index<BinaryValues> for BinaryValuesMultimap<N> {
    type Output = BTreeSet<N>;

    fn index(&self, index: BinaryValues) -> &Self::Output {
        &self.0[match index {
            BinaryValues::False => 0,
            BinaryValues::True => 1,
            BinaryValues::Both => 2,
        }]
    }
}

impl<N: Ord> IndexMut<BinaryValues> for BinaryValuesMultimap<N> {
    fn index_mut(&mut self, index: BinaryValues) -> &mut Self::Output {
        &mut self.0[match index {
            BinaryValues::False => 0,
            BinaryValues::True => 1,
            BinaryValues::Both => 2,
        }]
    }
}
