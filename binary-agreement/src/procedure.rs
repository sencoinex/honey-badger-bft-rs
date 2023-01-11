use crate::{
    binary_values::BinaryValues,
    coin_name::CoinName,
    epoch::Epoch,
    message::*,
    node::{NodeId, NodeMessage},
    session::SessionId,
    state::{BinaryAgreementState, FaultLog, FaultType},
    validator::{ValidatorIndex, ValidatorKeyShares, ValidatorSet},
    Error, Result,
};
use core::fmt;
use std::collections::BTreeMap;
use threshold_crypto::SignatureShare;

pub trait BinaryAgreement: fmt::Debug {
    type NodeId: NodeId;
    type ValidatorIndex: ValidatorIndex;
    type SessionId: SessionId;
    fn my_id(&self) -> &Self::NodeId;
    /// fetch message from received message queue whose epoch is equivalent with the arguments
    fn next_message(&mut self, epoch: &Epoch) -> NodeMessage<Self::NodeId>;
    fn send_message(&self, target_id: Self::NodeId, message: BinaryAgreementMessage);
    fn on_next_epoch(&mut self, epoch: &Epoch);
    fn handle_terminate_message(&mut self) {
        println!("{:?} has just detected terminate message.", self);
    }

    /// start binary agreement procedure
    fn propose(
        &mut self,
        input: bool,
        validator_set: ValidatorSet<Self::NodeId, Self::ValidatorIndex>,
        validator_key_shares: ValidatorKeyShares,
        session_id: Self::SessionId,
    ) -> Result<BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>> {
        let mut state = BinaryAgreementState::new(validator_set, validator_key_shares, session_id);
        self.on_start_new_epoch(input, &mut state)?;
        loop {
            let node_message = self.next_message(state.epoch());
            match node_message {
                NodeMessage::Terminate => {
                    self.handle_terminate_message();
                    break;
                }
                NodeMessage::BinaryAgreementMessage {
                    sender_id,
                    message: BinaryAgreementMessage { epoch, content },
                } => {
                    if !state.validator_set().contains(&sender_id) {
                        state.push_fault_log(FaultLog {
                            sender_id: sender_id.clone(),
                            message: BinaryAgreementMessage { epoch, content },
                            fault_type: FaultType::UnknownSender,
                        });
                        continue;
                    }
                    if epoch != *state.epoch() {
                        state.push_fault_log(FaultLog {
                            sender_id: sender_id.clone(),
                            message: BinaryAgreementMessage { epoch, content },
                            fault_type: FaultType::EpochMismatched {
                                current_epoch: *state.epoch(),
                                incoming_epoch: epoch,
                            },
                        });
                        continue;
                    }
                    match content {
                        BinaryAgreementMessageContent::BVal(message) => {
                            self.handle_bval(&sender_id, epoch, message, &mut state)?;
                        }
                        BinaryAgreementMessageContent::Aux(message) => {
                            self.handle_aux(&sender_id, epoch, message, &mut state)?;
                        }
                        BinaryAgreementMessageContent::Conf(message) => {
                            self.handle_conf(&sender_id, epoch, message, &mut state)?;
                        }
                        BinaryAgreementMessageContent::Coin(message) => {
                            self.handle_coin(&sender_id, epoch, message, &mut state)?;
                        }
                    }
                }
            }
            if let Some(coin_output) = state.get_coin_output() {
                let conf_output = state.get_conf_output().values();
                if let Some(single_conf_value) = conf_output.single() {
                    if single_conf_value == coin_output {
                        state.set_output(coin_output);
                        break;
                    } else {
                        // update epoch & start next round with
                        state.increment_epoch();
                        self.on_next_epoch(state.epoch());
                        self.on_start_new_epoch(single_conf_value, &mut state)?;
                    }
                } else {
                    // update epoch & start next round with
                    state.increment_epoch();
                    self.on_next_epoch(state.epoch());
                    self.on_start_new_epoch(coin_output, &mut state)?;
                }
            }
        }
        Ok(state)
    }

    fn on_start_new_epoch(
        &self,
        estimate: bool,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<()> {
        state.set_estimated(estimate);
        // broadcast BVal message
        let _ = state.try_add_sent_bval(estimate);
        self.broadcast_bval_message(estimate, *state.epoch(), state.validators().clone())?;
        let _ = state.try_add_received_bval(estimate, self.my_id().clone());
        Ok(())
    }

    fn handle_bval(
        &self,
        sender_id: &Self::NodeId,
        epoch: Epoch,
        message: BValMessage,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<()> {
        let value = message.into_inner();
        if !state.try_add_received_bval(value, sender_id.clone()) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BinaryAgreementMessage {
                    epoch,
                    content: BinaryAgreementMessageContent::BVal(value.into()),
                },
                fault_type: FaultType::DuplicateBVal,
            });
            return Ok(());
        }
        let count = state.get_received_bval_count(value);
        let max_durable_faulty_size = state.validator_set().max_durable_faulty_size();

        // upon receiving BVal(value) messages from f + 1 nodes, if BVal(value) has not been sent,
        // multicast BVal(value).
        if count >= max_durable_faulty_size + 1 {
            if state.try_add_sent_bval(value) {
                self.broadcast_bval_message(value, epoch, state.validators().clone())?;
            }
        }

        // upon receiving BVal(value) messages from 2f + 1 nodes, update bin_values.
        if count >= 2 * max_durable_faulty_size + 1 {
            if state.try_update_bin_values(value) {
                // multicast Aux(value)
                self.broadcast_aux_message(value, epoch, state.validators().clone())?;
                self.handle_aux(self.my_id(), epoch, AuxMessage::from(value), state)?;
            }
        }

        Ok(())
    }

    fn handle_aux(
        &self,
        sender_id: &Self::NodeId,
        epoch: Epoch,
        message: AuxMessage,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<()> {
        let value = message.into_inner();
        if !state.try_add_received_aux(value, sender_id.clone()) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BinaryAgreementMessage {
                    epoch,
                    content: BinaryAgreementMessageContent::Aux(value.into()),
                },
                fault_type: FaultType::DuplicateAux,
            });
            return Ok(());
        }
        if self.try_set_aux_output(state)? {
            // start conf phase by multicast Conf(aux output values)
            let aux_output = state.get_aux_output();
            let values = *aux_output.values();
            self.broadcast_conf_message(values, *state.epoch(), state.validators().clone())?;
            self.handle_conf(self.my_id(), epoch, ConfMessage::from(values), state)?;
        }
        Ok(())
    }

    fn try_set_aux_output(
        &self,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<bool> {
        if state.is_aux_decided() {
            return Ok(false);
        }
        let min_guarantee_size = state.validator_set().min_guarantee_size();
        let bin_values = state.get_bin_values();
        if bin_values.includes(BinaryValues::True)
            && state.get_received_aux_count(true) >= min_guarantee_size
        {
            state.set_aux_output(BinaryValues::True);
            Ok(true)
        } else if bin_values.includes(BinaryValues::False)
            && state.get_received_aux_count(false) >= min_guarantee_size
        {
            state.set_aux_output(BinaryValues::False);
            Ok(true)
        } else if bin_values.is_set() && state.get_total_received_aux_count() >= min_guarantee_size
        {
            state.set_aux_output(BinaryValues::Both);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn handle_conf(
        &self,
        sender_id: &Self::NodeId,
        epoch: Epoch,
        message: ConfMessage,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<()> {
        let values = message.into_inner();
        if !state.try_add_received_conf(values, sender_id.clone()) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BinaryAgreementMessage {
                    epoch,
                    content: BinaryAgreementMessageContent::Conf(ConfMessage::from(values)),
                },
                fault_type: FaultType::DuplicateConf,
            });
            return Ok(());
        }
        if self.try_set_conf_output(state)? {
            // start coin phase
            let coin_name = CoinName::new(state.session_id(), &epoch)?;
            let signature_share = state
                .validator_key_shares()
                .secret_key_share()
                .sign(coin_name);
            self.broadcast_coin_message(
                signature_share.clone(),
                *state.epoch(),
                state.validators().clone(),
            )?;
            self.handle_coin(
                self.my_id(),
                epoch,
                CommonCoinMessage::from(signature_share),
                state,
            )?;
        }
        Ok(())
    }

    fn handle_coin(
        &self,
        sender_id: &Self::NodeId,
        epoch: Epoch,
        message: CommonCoinMessage,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<()> {
        let signature_share = message.into_inner();
        // check message signature is valid
        let coin_name = CoinName::new(state.session_id(), &epoch)?;
        let node_index: u64 = *state
            .validator_set()
            .as_indices()
            .get(sender_id)
            .unwrap()
            .as_ref();
        let public_key_share = state
            .validator_key_shares()
            .get_public_key_share(node_index);
        let coin_name_hash = threshold_crypto::hasher::hash(coin_name);
        if !public_key_share.verify_with_hash(&signature_share, coin_name_hash) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BinaryAgreementMessage {
                    epoch,
                    content: BinaryAgreementMessageContent::Coin(CommonCoinMessage::from(
                        signature_share,
                    )),
                },
                fault_type: FaultType::InvalidSignatureShare,
            });
            return Ok(());
        }
        let _ = state.try_add_received_shares(signature_share, sender_id.clone());
        if state.is_coin_decided() {
            Ok(())
        } else {
            if state.get_total_received_shares_count()
                > state.validator_set().max_durable_faulty_size()
            {
                // compute signature
                let public_key_shares = state.validator_key_shares().public_key_shares();
                let shares = state.get_received_shares();
                let signature = public_key_shares.combine_signatures(shares.into_iter())?;
                let master_public_key = state
                    .validator_key_shares()
                    .public_key_shares()
                    .public_key();
                if master_public_key.verify_with_hash(&signature, coin_name_hash) {
                    state.set_coin_output(signature.parity());
                } else {
                    return Err(Error::InvalidCombinedSignature);
                }
            }
            Ok(())
        }
    }

    fn try_set_conf_output(
        &self,
        state: &mut BinaryAgreementState<Self::NodeId, Self::ValidatorIndex, Self::SessionId>,
    ) -> Result<bool> {
        if state.is_conf_decided() {
            return Ok(false);
        }
        if !state.is_aux_decided() {
            // case if aux phase has not been completed.
            return Ok(false);
        }
        let min_guarantee_size = state.validator_set().min_guarantee_size();
        let bin_values = state.get_bin_values();

        if bin_values.includes(BinaryValues::True)
            && state.get_received_conf_count(BinaryValues::True) >= min_guarantee_size
        {
            state.set_conf_output(BinaryValues::True);
            Ok(true)
        } else if bin_values.includes(BinaryValues::False)
            && state.get_received_conf_count(BinaryValues::False) >= min_guarantee_size
        {
            state.set_conf_output(BinaryValues::False);
            Ok(true)
        } else if bin_values.is_set() && state.get_total_received_conf_count() >= min_guarantee_size
        {
            state.set_conf_output(BinaryValues::Both);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn broadcast_bval_message(
        &self,
        value: bool,
        epoch: Epoch,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _index) in validators {
            if node_id != *self.my_id() {
                self.send_message(
                    node_id,
                    BinaryAgreementMessage {
                        epoch,
                        content: BinaryAgreementMessageContent::BVal(BValMessage::from(value)),
                    },
                );
            }
        }
        Ok(())
    }

    fn broadcast_aux_message(
        &self,
        value: bool,
        epoch: Epoch,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _index) in validators {
            if node_id != *self.my_id() {
                self.send_message(
                    node_id,
                    BinaryAgreementMessage {
                        epoch,
                        content: BinaryAgreementMessageContent::Aux(AuxMessage::from(value)),
                    },
                );
            }
        }
        Ok(())
    }

    fn broadcast_conf_message(
        &self,
        values: BinaryValues,
        epoch: Epoch,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _index) in validators {
            if node_id != *self.my_id() {
                self.send_message(
                    node_id,
                    BinaryAgreementMessage {
                        epoch,
                        content: BinaryAgreementMessageContent::Conf(ConfMessage::from(values)),
                    },
                );
            }
        }
        Ok(())
    }

    fn broadcast_coin_message(
        &self,
        signature_share: SignatureShare,
        epoch: Epoch,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _index) in validators {
            if node_id != *self.my_id() {
                self.send_message(
                    node_id,
                    BinaryAgreementMessage {
                        epoch,
                        content: BinaryAgreementMessageContent::Coin(CommonCoinMessage::from(
                            signature_share.clone(),
                        )),
                    },
                );
            }
        }
        Ok(())
    }
}
