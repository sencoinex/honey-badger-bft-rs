mod public_key_share;
mod public_key_shares;
pub use public_key_share::PublicKeyShare;
pub use public_key_shares::PublicKeyShares;

use crate::{hasher, Ciphertext, Signature};
use bls12_381::{pairing, G1Affine, G1Projective, G2Affine, Scalar};
use core::{fmt, hash};
use group::{ff::Field, Curve};
use hex_fmt::HexFmt;
use rand::{rngs::OsRng, Rng};
use std::cmp::Ordering;
use std::ops::Mul;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PublicKey(G1Projective);

impl PublicKey {
    pub fn new(projective: G1Projective) -> Self {
        Self(projective)
    }

    pub fn verify_with_hash(&self, sig: &Signature, hash: G2Affine) -> bool {
        let g1 = self.as_ref().into();
        let g2 = hash;
        let q = G1Affine::generator();
        let sig = sig.as_ref().into();
        // right: e(Pubkey, hash(m)) = e(s*Q, hash(m)) = e(Q, hash(m))**s
        // left : e(Q, Signature) = e(Q, s*hash(m)) = e(Q, hash(m))**s
        pairing(&g1, &g2) == pairing(&q, &sig)
    }

    pub fn verify<M: AsRef<[u8]>>(&self, sig: &Signature, msg: M) -> bool {
        self.verify_with_hash(sig, hasher::hash(msg))
    }

    pub fn encrypt<M: AsRef<[u8]>>(&self, msg: M) -> Ciphertext {
        self.encrypt_with_rng(&mut OsRng::default(), msg)
    }

    pub fn encrypt_with_rng<R: Rng, M: AsRef<[u8]>>(&self, rng: &mut R, msg: M) -> Ciphertext {
        let r = Scalar::random(rng);
        let u = G1Affine::generator().mul(r);
        let v: Vec<u8> = {
            let g = self.0.to_affine().mul(r);
            hasher::xor_with_hash(g.to_affine(), msg.as_ref())
        };
        let w = hasher::hash_with_g1(u.to_affine(), &v).mul(r);
        Ciphertext::new(u, v, w)
    }
}

impl AsRef<G1Projective> for PublicKey {
    fn as_ref(&self) -> &G1Projective {
        &self.0
    }
}

impl hash::Hash for PublicKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.to_affine().to_compressed().as_ref().hash(state)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let affine: G1Affine = self.0.to_affine();
        let uncompressed = affine.to_uncompressed();
        write!(f, "PublicKey({:0.10})", HexFmt(uncompressed))
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for PublicKey {
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
