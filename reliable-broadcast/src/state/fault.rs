use crate::{message::BroadcastMessage, node::NodeId};

#[derive(Debug)]
pub enum FaultType {
    UnknownSender,
    ReceivedValueFromNonProposer,
    MultipleValueMessages,
    MultipleEchoMessages,
    MultipleReadyMessages,
    InvalidProof,
}

pub struct FaultLog<ID: NodeId> {
    pub sender_id: ID,
    pub message: BroadcastMessage,
    pub fault_type: FaultType,
}
