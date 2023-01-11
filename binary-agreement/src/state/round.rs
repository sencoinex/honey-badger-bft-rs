use crate::{
    binary_values::{BinaryValueMultimap, BinaryValueSet, BinaryValues, BinaryValuesMultimap},
    node::NodeId,
};
use std::collections::BTreeMap;
use threshold_crypto::SignatureShare;

pub struct RoundState<NID: NodeId> {
    /// BVal message that we have already sent
    sent_bval: BinaryValueSet,
    /// received BVal messages
    received_bval: BinaryValueMultimap<NID>,
    /// The set of values for which `2 * f + 1 BVal`s have been received.
    bin_values: BinaryValueSet,
    /// received Aux messages
    received_aux: BinaryValueMultimap<NID>,
    /// output of Aux phase
    aux_output: BinaryValueSet,
    /// received Conf messages
    received_conf: BinaryValuesMultimap<NID>,
    /// output of Conf phase
    conf_output: BinaryValueSet,
    /// All received threshold signature shares.
    received_shares: BTreeMap<NID, SignatureShare>,
    /// output of common coin
    coin_output: Option<bool>,
}

impl<NID: NodeId> RoundState<NID> {
    pub fn new() -> Self {
        Self {
            sent_bval: BinaryValueSet::default(),
            received_bval: BinaryValueMultimap::default(),
            bin_values: BinaryValueSet::default(),
            received_aux: BinaryValueMultimap::default(),
            aux_output: BinaryValueSet::default(),
            received_conf: BinaryValuesMultimap::default(),
            conf_output: BinaryValueSet::default(),
            received_shares: BTreeMap::new(),
            coin_output: None,
        }
    }

    pub fn try_add_sent_bval(&mut self, value: bool) -> bool {
        self.sent_bval.insert(value)
    }

    pub fn try_add_received_bval(&mut self, value: bool, sender_id: NID) -> bool {
        self.received_bval[value].insert(sender_id)
    }

    pub fn get_received_bval_count(&self, value: bool) -> usize {
        self.received_bval[value].len()
    }

    pub fn bin_values(&self) -> &BinaryValueSet {
        &self.bin_values
    }

    pub fn try_update_bin_values(&mut self, value: bool) -> bool {
        self.bin_values.insert(value)
    }

    pub fn try_add_received_aux(&mut self, value: bool, sender_id: NID) -> bool {
        self.received_aux[value].insert(sender_id)
    }

    pub fn get_received_aux_count(&self, value: bool) -> usize {
        self.received_aux[value].len()
    }

    pub fn get_total_received_aux_count(&self) -> usize {
        let mut count = 0;
        count += self.received_aux[true].len();
        count += self.received_aux[false].len();
        count
    }

    pub fn is_aux_decided(&self) -> bool {
        self.aux_output.is_set()
    }

    pub fn get_aux_output(&self) -> &BinaryValueSet {
        &self.aux_output
    }

    pub fn set_aux_output(&mut self, values: BinaryValues) {
        self.aux_output = BinaryValueSet::new(values)
    }

    pub fn try_add_received_conf(&mut self, values: BinaryValues, sender_id: NID) -> bool {
        self.received_conf[values].insert(sender_id)
    }

    pub fn get_received_conf_count(&self, values: BinaryValues) -> usize {
        self.received_conf[values].len()
    }

    pub fn get_total_received_conf_count(&self) -> usize {
        let mut count = 0;
        if self.bin_values.includes(BinaryValues::True) {
            count += self.received_conf[BinaryValues::True].len();
        }
        if self.bin_values.includes(BinaryValues::False) {
            count += self.received_conf[BinaryValues::False].len();
        }
        if self.bin_values.includes(BinaryValues::Both) {
            count += self.received_conf[BinaryValues::Both].len();
        }
        count
    }

    pub fn is_conf_decided(&self) -> bool {
        self.conf_output.is_set()
    }

    pub fn get_conf_output(&self) -> &BinaryValueSet {
        &self.conf_output
    }

    pub fn set_conf_output(&mut self, values: BinaryValues) {
        self.conf_output = BinaryValueSet::new(values)
    }

    pub fn try_add_received_shares(
        &mut self,
        value: SignatureShare,
        sender_id: NID,
    ) -> Option<SignatureShare> {
        self.received_shares.insert(sender_id, value)
    }

    pub fn get_total_received_shares_count(&self) -> usize {
        self.received_shares.len()
    }

    pub fn received_shares(&self) -> &BTreeMap<NID, SignatureShare> {
        &self.received_shares
    }

    pub fn is_coin_decided(&self) -> bool {
        self.coin_output.is_some()
    }

    pub fn get_coin_output(&self) -> Option<bool> {
        self.coin_output
    }

    pub fn set_coin_output(&mut self, value: bool) {
        self.coin_output = Some(value)
    }
}
