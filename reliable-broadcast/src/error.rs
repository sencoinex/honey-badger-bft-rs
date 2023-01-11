use anyhow::anyhow;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("ReedSolomonError: {cause}")]
    ReedSolomonError { cause: anyhow::Error },

    #[error("Merkle tree dose not have payload length bytes.")]
    MerkleTreeMissingPayloadLength,

    #[error("Computed merkle tree root hash is invalid.")]
    IllegalMerkleTreeRootHash,
}

impl From<reed_solomon_erasure::Error> for Error {
    fn from(cause: reed_solomon_erasure::Error) -> Self {
        Self::ReedSolomonError {
            cause: anyhow!(cause),
        }
    }
}
