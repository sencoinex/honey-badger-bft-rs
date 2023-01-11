use super::Digest;
use tiny_keccak::{Hasher, Sha3};

/// Takes a chunk of one or two digests. In the former case, returns the digest itself, in the
/// latter, it returns the hash of the two digests.
pub(crate) fn hash_chunk(chunk: &[Digest]) -> Digest {
    if chunk.len() == 1 {
        chunk[0]
    } else {
        hash_pair(&chunk[0], &chunk[1])
    }
}

/// Returns the SHA-256 hash of the value's `[u8]` representation.
pub(crate) fn hash<T: AsRef<[u8]>>(value: T) -> Digest {
    let mut sha3 = Sha3::v256();
    sha3.update(value.as_ref());

    let mut out = [0u8; 32];
    sha3.finalize(&mut out);
    out
}

/// Returns the hash of the concatenated bytes of `d0` and `d1`.
pub(crate) fn hash_pair<T0: AsRef<[u8]>, T1: AsRef<[u8]>>(v0: &T0, v1: &T1) -> Digest {
    let bytes: Vec<u8> = v0.as_ref().iter().chain(v1.as_ref()).cloned().collect();
    hash(&bytes)
}
