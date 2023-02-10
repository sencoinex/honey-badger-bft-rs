use super::SecretKey;
use crate::{Ciphertext, DecryptionShare, SignatureShare};
use group::Curve;
use std::ops::Mul;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct SecretKeyShare(SecretKey);

impl SecretKeyShare {
    pub(crate) fn new(secret_key: SecretKey) -> Self {
        Self(secret_key)
    }

    pub fn into_inner(self) -> SecretKey {
        self.0
    }

    pub fn sign<M: AsRef<[u8]>>(&self, msg: M) -> SignatureShare {
        SignatureShare::new(self.0.sign(msg))
    }

    pub fn decrypt_share(&self, ct: &Ciphertext) -> Option<DecryptionShare> {
        if !ct.verify() {
            return None;
        }
        let g1_affine = ct.as_g1().to_affine();
        Some(DecryptionShare::new(g1_affine.mul(*self.0)))
    }

    pub fn decrypt_share_force(&self, ct: &Ciphertext) -> DecryptionShare {
        let g1_affine = ct.as_g1().to_affine();
        DecryptionShare::new(g1_affine.mul(*self.0))
    }
}
