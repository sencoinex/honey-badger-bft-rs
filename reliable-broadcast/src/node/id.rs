use core::{fmt, hash};

/// A peer node's unique identifier.
pub trait NodeId: Eq + Ord + Clone + fmt::Debug + hash::Hash + Send + Sync {}
impl<ID> NodeId for ID where ID: Eq + Ord + Clone + fmt::Debug + hash::Hash + Send + Sync {}
