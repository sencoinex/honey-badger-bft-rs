use crate::binary_values::BinaryValues;
use threshold_crypto::SignatureShare;

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryAgreementMessageContent {
    BVal(BValMessage),
    Aux(AuxMessage),
    Conf(ConfMessage),
    Coin(CommonCoinMessage),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BValMessage(bool);

impl BValMessage {
    pub fn into_inner(self) -> bool {
        self.0
    }
}

impl AsRef<bool> for BValMessage {
    fn as_ref(&self) -> &bool {
        &self.0
    }
}

impl From<bool> for BValMessage {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuxMessage(bool);

impl AuxMessage {
    pub fn into_inner(self) -> bool {
        self.0
    }
}

impl AsRef<bool> for AuxMessage {
    fn as_ref(&self) -> &bool {
        &self.0
    }
}

impl From<bool> for AuxMessage {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConfMessage(BinaryValues);

impl ConfMessage {
    pub fn into_inner(self) -> BinaryValues {
        self.0
    }
}

impl AsRef<BinaryValues> for ConfMessage {
    fn as_ref(&self) -> &BinaryValues {
        &self.0
    }
}

impl From<BinaryValues> for ConfMessage {
    fn from(value: BinaryValues) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommonCoinMessage(SignatureShare);

impl CommonCoinMessage {
    pub fn into_inner(self) -> SignatureShare {
        self.0
    }
}

impl AsRef<SignatureShare> for CommonCoinMessage {
    fn as_ref(&self) -> &SignatureShare {
        &self.0
    }
}

impl From<SignatureShare> for CommonCoinMessage {
    fn from(value: SignatureShare) -> Self {
        Self(value)
    }
}
