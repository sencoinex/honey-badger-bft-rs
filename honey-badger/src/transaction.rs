use core::{fmt::Debug, hash::Hash};

pub trait Transaction: Eq + Ord + Hash + Debug + Send + Sync {}
impl<TX> Transaction for TX where TX: Eq + Ord + Hash + Debug + Send + Sync {}

pub trait BatchTransactions: AsRef<[Self::Transaction]> {
    type Err;
    type Transaction: Transaction;
    fn serialize(&self) -> Result<Vec<u8>, Self::Err>;
}
