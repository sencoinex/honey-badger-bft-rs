use bls12_381::G1Projective;
use core::{fmt, hash};
use group::{Curve, Group};
use rand::distributions::{Distribution, Standard};
use rand::Rng;

/// A decryption share. A threshold of decryption shares can be used to decrypt a message.
#[derive(Clone, PartialEq, Eq)]
pub struct DecryptionShare(G1Projective);

impl DecryptionShare {
    pub fn new(projective: G1Projective) -> Self {
        Self(projective)
    }
}

impl Distribution<DecryptionShare> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DecryptionShare {
        DecryptionShare(G1Projective::random(rng))
    }
}

impl hash::Hash for DecryptionShare {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.to_affine().to_compressed().as_ref().hash(state);
    }
}

impl fmt::Debug for DecryptionShare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DecryptionShare...")
    }
}

impl AsRef<G1Projective> for DecryptionShare {
    fn as_ref(&self) -> &G1Projective {
        &self.0
    }
}
