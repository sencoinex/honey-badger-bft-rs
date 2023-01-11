use crate::{merkle::Proof, node::NodeId};
use std::collections::{BTreeMap, BTreeSet};

/// Reliable Broadcast State by Root Hash
pub struct RootHashState<ID: NodeId> {
    /// value message sender ID.
    proposer: Option<ID>,

    /// Whether we have already multicast `Echo`.
    echo_sent: bool,

    /// The proofs we have received via `Echo` messages, by sender ID.
    received_echo_messages: BTreeMap<ID, Proof<Vec<u8>>>,

    /// Whether we have already multicast `Ready`.
    ready_sent: bool,

    /// received sender ID set.
    received_ready_messages: BTreeSet<ID>,
}

impl<ID: NodeId> Default for RootHashState<ID> {
    fn default() -> Self {
        Self {
            proposer: None,
            echo_sent: false,
            received_echo_messages: BTreeMap::new(),
            ready_sent: false,
            received_ready_messages: BTreeSet::new(),
        }
    }
}

impl<ID: NodeId> RootHashState<ID> {
    pub fn get_proposer(&self) -> Option<&ID> {
        self.proposer.as_ref()
    }

    pub fn set_proposer(&mut self, proposer_id: ID) {
        self.proposer = Some(proposer_id)
    }

    pub fn is_echo_sent(&self) -> bool {
        self.echo_sent
    }

    pub fn turn_echo_sent_on(&mut self) {
        self.echo_sent = true
    }

    pub fn get_received_echo_message(&self, node_id: &ID) -> Option<&Proof<Vec<u8>>> {
        self.received_echo_messages.get(node_id)
    }

    pub fn insert_received_echo_message(&mut self, node_id: ID, proof: Proof<Vec<u8>>) {
        self.received_echo_messages.insert(node_id, proof);
    }

    pub fn count_received_echo_messages(&self) -> usize {
        self.received_echo_messages.len()
    }

    pub fn is_ready_sent(&self) -> bool {
        self.ready_sent
    }

    pub fn turn_ready_sent_on(&mut self) {
        self.ready_sent = true
    }

    pub fn insert_received_ready_message(&mut self, node_id: ID) -> bool {
        self.received_ready_messages.insert(node_id)
    }

    pub fn count_received_ready_messages(&self) -> usize {
        self.received_ready_messages.len()
    }
}
