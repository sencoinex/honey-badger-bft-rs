mod fault;
mod root_hash;
pub use fault::*;
pub use root_hash::RootHashState;

use crate::{
    encode::Coder,
    merkle::{Digest, Proof},
    node::NodeId,
    validator::{ValidatorIndex, ValidatorSet},
};
use std::collections::BTreeMap;

pub struct ReliableBroadcastState<ID: NodeId, IDX: ValidatorIndex> {
    /// validators
    validator_set: ValidatorSet<ID, IDX>,

    root_hash_states: BTreeMap<Digest, RootHashState<ID>>,

    fault_logs: Vec<FaultLog<ID>>,

    output: Option<Vec<u8>>,
}

impl<ID: NodeId, IDX: ValidatorIndex> ReliableBroadcastState<ID, IDX> {
    pub fn new(validator_set: ValidatorSet<ID, IDX>) -> Self {
        Self {
            validator_set,
            root_hash_states: BTreeMap::new(),
            fault_logs: Vec::new(),
            output: None,
        }
    }

    pub fn encoder(&self) -> &Coder {
        self.validator_set.as_encoder()
    }

    pub fn validator_set(&self) -> &ValidatorSet<ID, IDX> {
        &self.validator_set
    }

    pub fn validators(&self) -> &BTreeMap<ID, IDX> {
        self.validator_set.as_indices()
    }

    pub fn get_root_hash_state(&self, root_hash: &Digest) -> &RootHashState<ID> {
        self.root_hash_states
            .get(root_hash)
            .expect("root hash state must be initialized...")
    }

    pub fn get_or_init_mut_root_hash_state(&mut self, root_hash: Digest) -> &mut RootHashState<ID> {
        self.root_hash_states
            .entry(root_hash)
            .or_insert_with(RootHashState::default)
    }

    pub fn count_echo_messages(&self, digest: &Digest) -> usize {
        self.get_root_hash_state(digest)
            .count_received_echo_messages()
    }

    pub fn count_ready_messages(&self, digest: &Digest) -> usize {
        self.get_root_hash_state(digest)
            .count_received_ready_messages()
    }

    pub fn validate_proof(&self, proof: &Proof<Vec<u8>>, node_id: &ID) -> bool {
        self.validator_set.index(node_id).map(Into::into) == Some(proof.index())
            && proof.validate(self.validator_set.size())
    }

    pub fn can_compute_output(&self, root_hash: &Digest) -> bool {
        self.count_ready_messages(root_hash) > 2 * self.validator_set.max_durable_faulty_size()
            && self.count_echo_messages(root_hash)
                >= self.validator_set.as_encoder().data_shard_count()
    }

    pub fn fault_logs(&self) -> &Vec<FaultLog<ID>> {
        &self.fault_logs
    }

    pub fn push_fault_log(&mut self, fault_log: FaultLog<ID>) {
        self.fault_logs.push(fault_log);
    }

    pub fn is_decided(&self) -> bool {
        self.output.is_some()
    }

    pub fn set_output(&mut self, value: Vec<u8>) {
        self.output = Some(value)
    }

    pub fn get_output(&self) -> Option<&Vec<u8>> {
        self.output.as_ref()
    }
}
