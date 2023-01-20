use crate::node::NodeId;
use binary_agreement::BinaryAgreementState;
use reliable_broadcast::ReliableBroadcastState;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct AsynchronousCommonSubsetState<ID: NodeId> {
    reliable_broadcast_outputs: BTreeMap<ID, Option<Vec<u8>>>,
    reliable_broadcast_fault_logs: BTreeMap<ID, Vec<reliable_broadcast::FaultLog<ID>>>,
    binary_agreement_inputs: BTreeMap<ID, Option<bool>>,
    binary_agreement_outputs: BTreeMap<ID, Option<bool>>,
    binary_agreement_fault_logs: BTreeMap<ID, Vec<binary_agreement::FaultLog<ID>>>,
}

impl<ID: NodeId> AsynchronousCommonSubsetState<ID> {
    pub(crate) fn new() -> Self {
        Self {
            reliable_broadcast_outputs: BTreeMap::default(),
            reliable_broadcast_fault_logs: BTreeMap::default(),
            binary_agreement_inputs: BTreeMap::default(),
            binary_agreement_outputs: BTreeMap::default(),
            binary_agreement_fault_logs: BTreeMap::default(),
        }
    }

    pub fn into_output(self) -> BTreeMap<ID, Option<Vec<u8>>> {
        self.reliable_broadcast_outputs
    }

    pub fn as_reliable_broadcast_outputs(&self) -> &BTreeMap<ID, Option<Vec<u8>>> {
        &self.reliable_broadcast_outputs
    }

    pub(crate) fn set_reliable_broadcast_state<
        IDX: reliable_broadcast::validator::ValidatorIndex,
    >(
        &mut self,
        node_id: ID,
        state: ReliableBroadcastState<ID, IDX>,
    ) {
        let (output, fault_logs) = state.into_output_and_logs();
        self.set_reliable_broadcast_output(node_id.clone(), output);
        self.set_reliable_broadcast_fault_logs(node_id.clone(), fault_logs);
    }

    fn set_reliable_broadcast_output(&mut self, node_id: ID, output: Option<Vec<u8>>) {
        self.reliable_broadcast_outputs.insert(node_id, output);
    }

    fn set_reliable_broadcast_fault_logs(
        &mut self,
        node_id: ID,
        fault_logs: Vec<reliable_broadcast::FaultLog<ID>>,
    ) {
        self.reliable_broadcast_fault_logs
            .insert(node_id, fault_logs);
    }

    pub(crate) fn set_binary_agreement_input(&mut self, node_id: ID, input: Option<bool>) {
        self.binary_agreement_inputs.insert(node_id, input);
    }

    pub(crate) fn has_binary_agreement_input(&self, node_id: &ID) -> bool {
        self.binary_agreement_inputs.contains_key(node_id)
    }

    pub(crate) fn as_binary_agreement_outputs(&self) -> &BTreeMap<ID, Option<bool>> {
        &self.binary_agreement_outputs
    }

    pub(crate) fn set_binary_agreement_state<
        IDX: binary_agreement::validator::ValidatorIndex,
        SID: binary_agreement::session::SessionId,
    >(
        &mut self,
        node_id: ID,
        state: BinaryAgreementState<ID, IDX, SID>,
    ) {
        let (output, fault_logs) = state.into_output_and_logs();
        self.set_binary_agreement_output(node_id.clone(), output);
        self.set_binary_agreement_fault_logs(node_id.clone(), fault_logs);
    }

    fn set_binary_agreement_output(&mut self, node_id: ID, output: Option<bool>) {
        self.binary_agreement_outputs.insert(node_id, output);
    }

    fn set_binary_agreement_fault_logs(
        &mut self,
        node_id: ID,
        fault_logs: Vec<binary_agreement::FaultLog<ID>>,
    ) {
        self.binary_agreement_fault_logs.insert(node_id, fault_logs);
    }

    pub(crate) fn sum_binary_agreement_output(&self) -> usize {
        let mut result = 0;
        for (_, output) in &self.binary_agreement_outputs {
            if let Some(b) = output {
                if *b {
                    result += 1;
                }
            }
        }
        result
    }
}
