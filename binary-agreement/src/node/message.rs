use crate::message::BinaryAgreementMessage;
use crate::node::NodeId;

#[derive(Clone, PartialEq)]
pub enum NodeMessage<ID: NodeId> {
    BinaryAgreementMessage {
        sender_id: ID,
        message: BinaryAgreementMessage,
    },
    Terminate,
}
