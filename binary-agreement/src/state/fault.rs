use crate::{epoch::Epoch, message::BinaryAgreementMessage, node::NodeId};

#[derive(Debug, Clone)]
pub enum FaultType {
    UnknownSender,
    EpochMismatched {
        current_epoch: Epoch,
        incoming_epoch: Epoch,
    },
    DuplicateBVal,
    DuplicateAux,
    DuplicateConf,
    InvalidSignatureShare,
}

#[derive(Debug, Clone)]
pub struct FaultLog<NID: NodeId> {
    pub sender_id: NID,
    pub message: BinaryAgreementMessage,
    pub fault_type: FaultType,
}
