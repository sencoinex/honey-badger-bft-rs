pub(crate) mod hasher;
mod merkle_tree;
mod proof;

pub use merkle_tree::MerkleTree;
pub use proof::Proof;

pub type Digest = [u8; 32];
