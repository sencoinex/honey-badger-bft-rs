use crate::{Epoch, NodeId};
use threshold_crypto::DecryptionShare;

#[derive(Debug, Clone, PartialEq)]
pub struct DecryptionShareMessage<ID: NodeId> {
    pub proposer_id: ID,
    pub epoch: Epoch,
    pub decryption_share: DecryptionShare,
}

impl<ID: NodeId> DecryptionShareMessage<ID> {
    pub fn new(proposer_id: ID, epoch: Epoch, decryption_share: DecryptionShare) -> Self {
        Self {
            proposer_id,
            epoch,
            decryption_share,
        }
    }
}
