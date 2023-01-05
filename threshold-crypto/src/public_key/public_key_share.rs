use super::PublicKey;
use crate::{hasher, Ciphertext, DecryptionShare, SignatureShare};
use bls12_381::{pairing, G2Affine};
use core::fmt;
use group::Curve;
use hex_fmt::HexFmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PublicKeyShare(PublicKey);

impl PublicKeyShare {
    pub(crate) fn new(public_key: PublicKey) -> Self {
        Self(public_key)
    }

    pub fn into_inner(self) -> PublicKey {
        self.0
    }

    pub fn verify_with_hash(&self, sig: &SignatureShare, hash: G2Affine) -> bool {
        self.0.verify_with_hash(sig.as_ref(), hash)
    }

    pub fn verify<M: AsRef<[u8]>>(&self, sig: &SignatureShare, msg: M) -> bool {
        self.0.verify(sig.as_ref(), msg)
    }

    pub fn verify_decryption_share(&self, share: &DecryptionShare, ct: &Ciphertext) -> bool {
        let g1 = ct.as_g1();
        let msg = ct.as_msg();
        let g2 = ct.as_g2();
        let hash = hasher::hash_with_g1(g1.to_affine(), msg);
        pairing(&share.as_ref().to_affine(), &hash)
            == pairing(&(self.0).as_ref().to_affine(), &g2.to_affine())
    }
}

impl fmt::Debug for PublicKeyShare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uncompressed = (self.0).0.to_affine().to_uncompressed();
        write!(f, "PublicKeyShare({:0.10})", HexFmt(uncompressed))
    }
}
