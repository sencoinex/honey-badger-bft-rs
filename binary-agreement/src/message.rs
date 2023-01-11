mod content;
pub use content::*;

use crate::epoch::Epoch;

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryAgreementMessage {
    pub epoch: Epoch,
    pub content: BinaryAgreementMessageContent,
}
