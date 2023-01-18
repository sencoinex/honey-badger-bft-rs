mod fault;
mod round;
pub use fault::*;
pub use round::*;

use crate::{
    binary_values::{BinaryValueSet, BinaryValues},
    epoch::Epoch,
    node::NodeId,
    session::SessionId,
    validator::{ValidatorIndex, ValidatorKeyShares, ValidatorSet},
};
use std::collections::BTreeMap;
use threshold_crypto::SignatureShare;

pub struct BinaryAgreementState<NID: NodeId, IDX: ValidatorIndex, SID: SessionId> {
    /// validators
    validator_set: ValidatorSet<NID, IDX>,

    /// key share
    validator_key_shares: ValidatorKeyShares,

    /// Session identifier, to prevent replaying messages in other instances.
    session_id: SID,

    epoch: Epoch,

    /// The estimate of the decision value in the current epoch.
    estimated: Option<bool>,

    rounds: BTreeMap<Epoch, RoundState<NID>>,

    fault_logs: Vec<FaultLog<NID>>,

    output: Option<bool>,
}

impl<NID: NodeId, IDX: ValidatorIndex, SID: SessionId> BinaryAgreementState<NID, IDX, SID> {
    pub fn new(
        validator_set: ValidatorSet<NID, IDX>,
        validator_key_shares: ValidatorKeyShares,
        session_id: SID,
    ) -> Self {
        Self {
            validator_set,
            validator_key_shares,
            session_id,
            epoch: Epoch::default(),
            estimated: None,
            rounds: BTreeMap::from([(Epoch::default(), RoundState::new())]),
            fault_logs: Vec::new(),
            output: None,
        }
    }

    pub fn validator_set(&self) -> &ValidatorSet<NID, IDX> {
        &self.validator_set
    }

    pub fn validators(&self) -> &BTreeMap<NID, IDX> {
        self.validator_set.as_indices()
    }

    pub fn validator_key_shares(&self) -> &ValidatorKeyShares {
        &self.validator_key_shares
    }

    pub fn session_id(&self) -> &SID {
        &self.session_id
    }

    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    pub fn increment_epoch(&mut self) {
        self.estimated = None;
        self.epoch.increment();
        self.rounds.insert(self.epoch, RoundState::new());
    }

    pub fn set_estimated(&mut self, value: bool) {
        self.estimated = Some(value);
    }

    pub fn current_round(&self) -> &RoundState<NID> {
        self.rounds
            .get(&self.epoch)
            .expect("round state must be initialized...")
    }

    pub fn mut_current_round(&mut self) -> &mut RoundState<NID> {
        self.rounds
            .get_mut(&self.epoch)
            .expect("round state must be initialized...")
    }

    pub fn try_add_sent_bval(&mut self, value: bool) -> bool {
        self.mut_current_round().try_add_sent_bval(value)
    }

    pub fn try_add_received_bval(&mut self, value: bool, sender_id: NID) -> bool {
        self.mut_current_round()
            .try_add_received_bval(value, sender_id)
    }

    pub fn get_received_bval_count(&self, value: bool) -> usize {
        self.current_round().get_received_bval_count(value)
    }

    pub fn get_bin_values(&self) -> &BinaryValueSet {
        self.current_round().bin_values()
    }

    pub fn try_update_bin_values(&mut self, value: bool) -> bool {
        self.mut_current_round().try_update_bin_values(value)
    }

    pub fn try_add_received_aux(&mut self, value: bool, sender_id: NID) -> bool {
        self.mut_current_round()
            .try_add_received_aux(value, sender_id)
    }

    pub fn get_received_aux_count(&self, value: bool) -> usize {
        self.current_round().get_received_aux_count(value)
    }

    pub fn get_total_received_aux_count(&self) -> usize {
        self.current_round().get_total_received_aux_count()
    }

    pub fn is_aux_decided(&self) -> bool {
        self.current_round().is_aux_decided()
    }

    pub fn get_aux_output(&self) -> &BinaryValueSet {
        self.current_round().get_aux_output()
    }

    pub fn set_aux_output(&mut self, values: BinaryValues) {
        self.mut_current_round().set_aux_output(values)
    }

    pub fn try_add_received_conf(&mut self, values: BinaryValues, sender_id: NID) -> bool {
        self.mut_current_round()
            .try_add_received_conf(values, sender_id)
    }

    pub fn get_received_conf_count(&self, values: BinaryValues) -> usize {
        self.current_round().get_received_conf_count(values)
    }

    pub fn get_total_received_conf_count(&self) -> usize {
        self.current_round().get_total_received_conf_count()
    }

    pub fn is_conf_decided(&self) -> bool {
        self.current_round().is_conf_decided()
    }

    pub fn get_conf_output(&self) -> &BinaryValueSet {
        self.current_round().get_conf_output()
    }

    pub fn set_conf_output(&mut self, values: BinaryValues) {
        self.mut_current_round().set_conf_output(values)
    }

    pub fn try_add_received_shares(
        &mut self,
        value: SignatureShare,
        sender_id: NID,
    ) -> Option<SignatureShare> {
        self.mut_current_round()
            .try_add_received_shares(value, sender_id)
    }

    pub fn get_total_received_shares_count(&self) -> usize {
        self.current_round().get_total_received_shares_count()
    }

    pub fn get_received_shares(&self) -> Vec<(u64, &SignatureShare)> {
        let received_shares = self.current_round().received_shares();
        received_shares
            .into_iter()
            .map(|(node_id, share)| {
                let index: u64 = self.validator_set.index(node_id).unwrap().into();
                (index, share)
            })
            .collect()
    }

    pub fn is_coin_decided(&self) -> bool {
        self.current_round().is_coin_decided()
    }

    pub fn get_coin_output(&self) -> Option<bool> {
        self.current_round().get_coin_output()
    }

    pub fn set_coin_output(&mut self, value: bool) {
        self.mut_current_round().set_coin_output(value)
    }

    pub fn fault_logs(&self) -> &Vec<FaultLog<NID>> {
        &self.fault_logs
    }

    pub fn push_fault_log(&mut self, fault_log: FaultLog<NID>) {
        self.fault_logs.push(fault_log);
    }

    pub fn is_decided(&self) -> bool {
        self.output.is_some()
    }

    pub fn set_output(&mut self, value: bool) {
        self.output = Some(value)
    }

    pub fn get_output(&self) -> Option<bool> {
        self.output
    }

    pub fn into_output_and_logs(self) -> (Option<bool>, Vec<FaultLog<NID>>) {
        (self.output, self.fault_logs)
    }
}
