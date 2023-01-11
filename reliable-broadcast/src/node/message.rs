use crate::message::BroadcastMessage;
use crate::node::NodeId;

#[derive(Clone, PartialEq)]
pub enum NodeMessage<ID: NodeId> {
    BroadcastMessage {
        sender_id: ID,
        message: BroadcastMessage,
    },
    Terminate,
}
