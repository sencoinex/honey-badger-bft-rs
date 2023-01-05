mod signature_share;
pub use signature_share::SignatureShare;

use bls12_381::{G2Affine, G2Projective};
use core::{fmt, hash};
use group::Curve;
use hex_fmt::HexFmt;
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Eq)]
pub struct Signature(G2Projective);

impl Signature {
    pub(crate) fn new(projective: G2Projective) -> Self {
        Self(projective)
    }

    pub fn into_inner(self) -> G2Projective {
        self.0
    }

    pub fn parity(&self) -> bool {
        let uncompressed = self.0.to_affine().to_uncompressed();
        let xor_bytes: u8 = uncompressed
            .as_ref()
            .iter()
            .fold(0, |result, byte| result ^ byte);
        let parity = 0 != xor_bytes.count_ones() % 2;
        parity
    }
}

impl AsRef<G2Projective> for Signature {
    fn as_ref(&self) -> &G2Projective {
        &self.0
    }
}

impl hash::Hash for Signature {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let affine: G2Affine = (&self.0).into();
        affine.to_compressed().as_ref().hash(state)
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let affine: G2Affine = (&self.0).into();
        let uncompressed = affine.to_uncompressed();
        write!(f, "Signature({:0.10})", HexFmt(uncompressed))
    }
}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else {
            let mine = self.0.to_affine().to_compressed();
            let others = other.0.to_affine().to_compressed();
            mine.as_ref().cmp(others.as_ref())
        }
    }
}
