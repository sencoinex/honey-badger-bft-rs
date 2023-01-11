use super::{hasher, Digest, Proof};
use core::{fmt, mem};
use hex_fmt::HexFmt;

pub struct MerkleTree<T: AsRef<[u8]>> {
    levels: Vec<Vec<Digest>>,
    values: Vec<T>,
    root_hash: Digest,
}

impl<T: AsRef<[u8]> + Clone> MerkleTree<T> {
    pub fn new(values: Vec<T>) -> Self {
        values.into()
    }

    /// Returns the proof for entry `index`, if that is a valid index.
    pub fn proof(&self, index: usize) -> Option<Proof<T>> {
        let value = self.values.get(index)?.clone();
        let mut lvl_i = index;
        let mut digests = Vec::new();
        for level in &self.levels {
            // Insert the sibling hash if there is one.
            if let Some(digest) = level.get(lvl_i ^ 1) {
                digests.push(*digest);
            }
            lvl_i /= 2;
        }
        Some(Proof::new(value, index, digests, self.root_hash))
    }

    /// Returns the root hash of the tree.
    pub fn root_hash(&self) -> &Digest {
        &self.root_hash
    }

    /// Returns a the slice containing all leaf values.
    pub fn values(&self) -> &[T] {
        &self.values
    }

    /// Consumes the tree, and returns the vector of leaf values.
    pub fn into_values(self) -> Vec<T> {
        self.values
    }
}

impl<T: AsRef<[u8]>> From<Vec<T>> for MerkleTree<T> {
    fn from(values: Vec<T>) -> Self {
        let mut levels = Vec::new();
        let mut cur_lvl: Vec<Digest> = values.iter().map(hasher::hash).collect();
        while cur_lvl.len() > 1 {
            let next_lvl = cur_lvl.chunks(2).map(hasher::hash_chunk).collect();
            levels.push(mem::replace(&mut cur_lvl, next_lvl));
        }
        let root_hash = cur_lvl[0];
        MerkleTree {
            levels,
            values,
            root_hash,
        }
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for MerkleTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MerkleTree {{ root_hash: {:0.10}, level: {}, .. }}",
            HexFmt(self.root_hash),
            self.levels.len()
        )
    }
}
