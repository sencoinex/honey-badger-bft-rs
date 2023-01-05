mod secret_key_share;
mod secret_key_shares;
pub use secret_key_share::SecretKeyShare;
pub use secret_key_shares::SecretKeyShares;

use crate::{hasher, Ciphertext, PublicKey, Signature};
use bls12_381::{G1Affine, Scalar};
use core::fmt;
use group::Curve;
use std::ops::{Deref, Mul};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, PartialEq, Eq)]
pub struct SecretKey(Scalar);

impl SecretKey {
    pub fn new(scalar: Scalar) -> Self {
        Self(scalar)
    }

    pub fn sign<M: AsRef<[u8]>>(&self, msg: M) -> Signature {
        let g2 = hasher::hash(msg);
        let signature = g2.mul(&self.0);
        Signature::new(signature)
    }

    pub fn decrypt(&self, ct: &Ciphertext) -> Option<Vec<u8>> {
        if !ct.verify() {
            return None;
        }
        let u = ct.as_g1();
        let v = ct.as_msg();
        let g = u.to_affine().mul(self.0);
        Some(hasher::xor_with_hash(g.to_affine(), v))
    }

    pub fn compute_public_key(&self) -> PublicKey {
        let q = G1Affine::generator();
        let projective = q.mul(&self.0);
        PublicKey::new(projective)
    }
}

impl Deref for SecretKey {
    type Target = Scalar;
    fn deref(&self) -> &Scalar {
        &self.0
    }
}

impl Default for SecretKey {
    fn default() -> Self {
        Self::new(Scalar::zero())
    }
}

/// suppress to display its value unintentionally
impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretKey...")
    }
}

impl Zeroize for SecretKey {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl ZeroizeOnDrop for SecretKey {}
