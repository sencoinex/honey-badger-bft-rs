use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Not enough signature shares")]
    NotEnoughShares,
    #[error("Signature shares contain a duplicated index")]
    DuplicateEntry,
}
