use crate::hasher;
use bls12_381::{pairing, G1Affine, G1Projective, G2Projective};
use core::hash;
use group::Curve;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Ciphertext(
    #[serde(with = "crate::serializers::g1_projective")] G1Projective,
    Vec<u8>,
    #[serde(with = "crate::serializers::g2_projective")] G2Projective,
);

impl Ciphertext {
    pub fn new(g1: G1Projective, msg: Vec<u8>, g2: G2Projective) -> Self {
        Self(g1, msg, g2)
    }

    pub fn as_g1(&self) -> &G1Projective {
        &self.0
    }

    pub fn as_msg(&self) -> &[u8] {
        &self.1.as_slice()
    }

    pub fn as_g2(&self) -> &G2Projective {
        &self.2
    }

    /// Returns `true` if this is a valid ciphertext. This check is necessary to prevent
    /// chosen-ciphertext attacks.
    pub fn verify(&self) -> bool {
        let Ciphertext(ref u, ref v, ref w) = *self;
        let hash = hasher::hash_with_g1(u.to_affine(), v);
        pairing(&G1Affine::generator(), &w.to_affine()) == pairing(&u.to_affine(), &hash)
    }
}

impl hash::Hash for Ciphertext {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Ciphertext(ref u, ref v, ref w) = *self;
        u.to_affine().to_compressed().as_ref().hash(state);
        v.hash(state);
        w.to_affine().to_compressed().as_ref().hash(state);
    }
}

impl PartialOrd for Ciphertext {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Ciphertext {
    fn cmp(&self, other: &Self) -> Ordering {
        let Ciphertext(ref u0, ref v0, ref w0) = self;
        let Ciphertext(ref u1, ref v1, ref w1) = other;

        let mine = u0.to_affine().to_compressed();
        let others = u1.to_affine().to_compressed();
        mine.as_ref()
            .cmp(others.as_ref())
            .then(v0.cmp(v1))
            .then_with(|| {
                let mine = w0.to_affine().to_compressed();
                let others = w1.to_affine().to_compressed();
                mine.as_ref().cmp(others.as_ref())
            })
    }
}
