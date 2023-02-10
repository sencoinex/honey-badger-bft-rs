use crate::{message::DecryptionShareMessage, node::NodeId};

#[derive(Debug, Clone)]
pub enum FaultLog<ID: NodeId> {
    ReliableBroadcast(reliable_broadcast::FaultLog<ID>),
    BinaryAgreement(binary_agreement::FaultLog<ID>),
    DecryptionShare(DecryptionShareFaultLog<ID>),
}

#[derive(Debug, Clone)]
pub enum DecryptionShareFaultType {
    UnknownSender,
    InvalidDecryptionShare,
}

#[derive(Debug, Clone)]
pub struct DecryptionShareFaultLog<ID: NodeId> {
    pub sender_id: ID,
    pub message: DecryptionShareMessage<ID>,
    pub fault_type: DecryptionShareFaultType,
}
