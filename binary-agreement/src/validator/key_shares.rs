use threshold_crypto::{PublicKeyShare, PublicKeyShares, SecretKeyShare};

/// TPKE.Setup information
#[derive(Clone, PartialEq, Eq)]
pub struct ValidatorKeyShares {
    secret_key_share: SecretKeyShare,
    public_key_shares: PublicKeyShares,
}

impl ValidatorKeyShares {
    pub fn new(secret_key_share: SecretKeyShare, public_key_shares: PublicKeyShares) -> Self {
        Self {
            secret_key_share,
            public_key_shares,
        }
    }

    pub fn secret_key_share(&self) -> &SecretKeyShare {
        &self.secret_key_share
    }

    pub fn public_key_shares(&self) -> &PublicKeyShares {
        &self.public_key_shares
    }

    pub fn get_public_key_share(&self, index: u64) -> PublicKeyShare {
        self.public_key_shares.public_key_share(index)
    }
}
