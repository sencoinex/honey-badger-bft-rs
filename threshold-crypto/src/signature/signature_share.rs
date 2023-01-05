use super::Signature;
use core::fmt;
use group::Curve;
use hex_fmt::HexFmt;

#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SignatureShare(Signature);

impl SignatureShare {
    pub(crate) fn new(signature: Signature) -> Self {
        Self(signature)
    }
}

impl fmt::Debug for SignatureShare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let uncompressed = (self.0).0.to_affine().to_uncompressed();
        write!(f, "SignatureShare({:0.10})", HexFmt(uncompressed))
    }
}

impl AsRef<Signature> for SignatureShare {
    fn as_ref(&self) -> &Signature {
        &self.0
    }
}
