use super::ValidatorIndex;
use crate::{node::NodeId, Result};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct ValidatorSet<ID: NodeId, IDX: ValidatorIndex> {
    indices: BTreeMap<ID, IDX>,
    max_durable_faulty_size: usize,
}

impl<ID: NodeId, IDX: ValidatorIndex> ValidatorSet<ID, IDX> {
    pub fn new(indices: BTreeMap<ID, IDX>) -> Result<Self> {
        let size = indices.len();
        let max_durable_faulty_size = (size - 1) / 3;
        Ok(Self {
            indices,
            max_durable_faulty_size,
        })
    }

    pub fn as_indices(&self) -> &BTreeMap<ID, IDX> {
        &self.indices
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn max_durable_faulty_size(&self) -> usize {
        self.max_durable_faulty_size
    }

    pub fn min_guarantee_size(&self) -> usize {
        self.size() - self.max_durable_faulty_size()
    }

    /// Returns `true` if the given ID belongs to a known validator.
    #[inline]
    pub fn contains(&self, id: &ID) -> bool {
        self.indices.contains_key(id)
    }

    /// Returns the validators index in the ordered list of all IDs.
    #[inline]
    pub fn index(&self, id: &ID) -> Option<IDX> {
        self.indices.get(id).map(|idx| *idx)
    }
}
