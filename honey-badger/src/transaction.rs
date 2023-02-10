use crate::Epoch;
use core::{fmt::Debug, hash::Hash};
use std::collections::BTreeSet;

pub trait Transaction: Eq + Ord + Hash + Debug + Send + Sync + TryFrom<Vec<u8>> {}
impl<TX> Transaction for TX where TX: Eq + Ord + Hash + Debug + Send + Sync + TryFrom<Vec<u8>> {}

pub trait BatchTransactions: AsRef<[Self::Transaction]> {
    type Err;
    type Transaction: Transaction;
    fn serialize(&self) -> Result<Vec<u8>, Self::Err>;
}

/// Verified transaction, it is often referred as a "block".
pub struct VerifiedTransactions<TX: Transaction> {
    pub epoch: Epoch,
    pub transactions: BTreeSet<TX>,
}

impl<TX: Transaction> VerifiedTransactions<TX> {
    pub fn new(epoch: Epoch) -> Self {
        Self {
            epoch,
            transactions: BTreeSet::default(),
        }
    }

    /// Adds a transaction to the set.
    ///
    /// Returns whether the transaction was newly added.
    pub fn add_transaction(&mut self, transaction: TX) -> bool {
        self.transactions.insert(transaction)
    }
}
