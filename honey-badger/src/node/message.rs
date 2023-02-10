use super::NodeId;
use crate::DecryptionShareMessage;

#[derive(Clone, PartialEq)]
pub enum NodeMessage<ID: NodeId> {
    BroadcastMessage {
        sender_id: ID,
        message: DecryptionShareMessage<ID>,
    },
    Terminate,
}
