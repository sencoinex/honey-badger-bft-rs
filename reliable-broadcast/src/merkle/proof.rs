use super::{hasher, Digest};
use core::fmt;
use hex_fmt::HexFmt;

#[derive(Clone, PartialEq)]
pub struct Proof<T: AsRef<[u8]>> {
    value: T,
    index: usize,
    digests: Vec<Digest>,
    root_hash: Digest,
}

impl<T: AsRef<[u8]>> Proof<T> {
    pub(crate) fn new(value: T, index: usize, digests: Vec<Digest>, root_hash: Digest) -> Self {
        Self {
            index,
            digests,
            value,
            root_hash,
        }
    }

    /// Returns `true` if the digests in this proof constitute a valid branch in a Merkle tree with
    /// the root hash.
    pub fn validate(&self, n: usize) -> bool {
        let mut digest = hasher::hash(&self.value);
        let mut lvl_i = self.index;
        let mut lvl_n = n;
        let mut digest_itr = self.digests.iter();
        while lvl_n > 1 {
            if lvl_i ^ 1 < lvl_n {
                digest = match digest_itr.next() {
                    None => return false, // Not enough levels in the proof.
                    Some(sibling) if lvl_i & 1 == 1 => hasher::hash_pair(&sibling, &digest),
                    Some(sibling) => hasher::hash_pair(&digest, &sibling),
                };
            }
            lvl_i /= 2; // Our index on the next level.
            lvl_n = (lvl_n + 1) / 2; // The next level's size.
        }
        if digest_itr.next().is_some() {
            return false; // Too many levels in the proof.
        }
        digest == self.root_hash
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn root_hash(&self) -> &Digest {
        &self.root_hash
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T: AsRef<[u8]>> fmt::Debug for Proof<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Proof {{ #{}, root_hash: {:0.10}, value: {:0.10}, .. }}",
            &self.index(),
            HexFmt(self.root_hash()),
            HexFmt(self.value())
        )
    }
}
