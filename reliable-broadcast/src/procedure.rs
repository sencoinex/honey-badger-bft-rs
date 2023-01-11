use crate::{
    encode::Coder,
    merkle::Proof,
    merkle::{Digest, MerkleTree},
    message::*,
    node::{NodeId, NodeMessage},
    state::{FaultLog, FaultType, ReliableBroadcastState},
    validator::{ValidatorIndex, ValidatorSet},
    Error, Result,
};
use byteorder::{BigEndian, ByteOrder};
use core::fmt;
use std::collections::BTreeMap;

pub trait ReliableBroadcast: fmt::Debug {
    type NodeId: NodeId;
    type ValidatorIndex: ValidatorIndex;
    fn my_id(&self) -> &Self::NodeId;
    fn next_message(&self) -> NodeMessage<Self::NodeId>;
    fn send_message(&self, target_id: Self::NodeId, message: BroadcastMessage);
    fn handle_terminate_message(&self) {
        println!("{:?} has just detected terminate message.", self);
    }
    fn handle_duplicated_value_message(&self, sender_id: &Self::NodeId, proof: &Proof<Vec<u8>>) {
        println!(
            "{:?} received Value({:?}) multiple times from {:?}.",
            self, proof, sender_id
        );
    }
    fn handle_duplicated_echo_message(&self, sender_id: &Self::NodeId, proof: &Proof<Vec<u8>>) {
        println!(
            "{:?} received Echo({:?}) multiple times from {:?}.",
            self, proof, sender_id
        );
    }
    fn handle_duplicated_ready_message(&self, sender_id: &Self::NodeId, proof: &Digest) {
        println!(
            "{:?} received Ready({:?}) multiple times from {:?}.",
            self, proof, sender_id
        );
    }

    fn propose(
        &self,
        input: Vec<u8>,
        validator_set: ValidatorSet<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>> {
        let encoder = validator_set.as_encoder();
        let shards = encode_to_shards(&encoder, input)?;
        let merkle_tree = MerkleTree::from(shards);
        assert_eq!(validator_set.size(), merkle_tree.values().len());
        let mut initial_value_message = None;
        for (node_id, index) in validator_set.as_indices().clone() {
            let proof = merkle_tree.proof(index.into()).unwrap();
            let value_message = ValueMessage::from(proof);
            if node_id == *self.my_id() {
                initial_value_message = Some(value_message);
            } else {
                self.send_message(node_id, BroadcastMessage::Value(value_message));
            }
        }
        self.execute(initial_value_message, validator_set)
    }

    /// execute reliable broadcast procedure
    fn execute(
        &self,
        initial_value_message: Option<ValueMessage>,
        validator_set: ValidatorSet<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>> {
        let mut state = ReliableBroadcastState::new(validator_set);
        if let Some(initial_value_message) = initial_value_message {
            self.handle_value(self.my_id(), initial_value_message, &mut state)?;
        }
        loop {
            let node_message = self.next_message();
            match node_message {
                NodeMessage::Terminate => {
                    self.handle_terminate_message();
                    break;
                }
                NodeMessage::BroadcastMessage { sender_id, message } => {
                    if !state.validator_set().contains(&sender_id) {
                        state.push_fault_log(FaultLog {
                            sender_id: sender_id.clone(),
                            message,
                            fault_type: FaultType::UnknownSender,
                        });
                        continue;
                    }
                    match message {
                        BroadcastMessage::Value(message) => {
                            self.handle_value(&sender_id, message, &mut state)?;
                        }
                        BroadcastMessage::Echo(message) => {
                            self.handle_echo(&sender_id, message, &mut state)?;
                        }
                        BroadcastMessage::Ready(message) => {
                            self.handle_ready(&sender_id, message, &mut state)?;
                        }
                    }
                    if state.is_decided() {
                        break;
                    }
                }
            }
        }
        Ok(state)
    }

    fn handle_value(
        &self,
        sender_id: &Self::NodeId,
        message: ValueMessage,
        state: &mut ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        let proof = message.as_ref();
        // validate proof first.
        if !state.validate_proof(proof, self.my_id()) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BroadcastMessage::Value(message),
                fault_type: FaultType::InvalidProof,
            });
            return Ok(());
        }

        let root_hash = message.as_ref().root_hash();
        let root_hash_state = state.get_or_init_mut_root_hash_state(*root_hash);
        if let Some(proposer) = root_hash_state.get_proposer() {
            // If we have already received value message
            return if proposer != sender_id {
                state.push_fault_log(FaultLog {
                    sender_id: sender_id.clone(),
                    message: BroadcastMessage::Value(message),
                    fault_type: FaultType::ReceivedValueFromNonProposer,
                });
                Ok(())
            } else {
                // validate proof
                let got_proof = root_hash_state.get_received_echo_message(sender_id);
                if got_proof == Some(proof) {
                    // if duplicated, just log warnings
                    self.handle_duplicated_value_message(sender_id, proof);
                } else {
                    state.push_fault_log(FaultLog {
                        sender_id: sender_id.clone(),
                        message: BroadcastMessage::Value(message),
                        fault_type: FaultType::MultipleValueMessages,
                    });
                };
                Ok(())
            };
        }

        root_hash_state.set_proposer(sender_id.clone());
        root_hash_state.turn_echo_sent_on();
        self.broadcast_echo_message(proof.clone(), state.validators().clone())?;
        Ok(())
    }

    fn handle_echo(
        &self,
        sender_id: &Self::NodeId,
        message: EchoMessage,
        state: &mut ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        let proof = message.as_ref();
        // validate proof first.
        if !state.validate_proof(proof, sender_id) {
            state.push_fault_log(FaultLog {
                sender_id: sender_id.clone(),
                message: BroadcastMessage::Echo(message),
                fault_type: FaultType::InvalidProof,
            });
            return Ok(());
        }

        let root_hash = proof.root_hash();
        let min_guarantee_size = state.validator_set().min_guarantee_size();
        let root_hash_state = state.get_or_init_mut_root_hash_state(*root_hash);

        if let Some(got_proof) = root_hash_state.get_received_echo_message(sender_id) {
            if got_proof == proof {
                // if duplicated, just log warnings
                self.handle_duplicated_echo_message(sender_id, proof);
            } else {
                state.push_fault_log(FaultLog {
                    sender_id: sender_id.clone(),
                    message: BroadcastMessage::Echo(message),
                    fault_type: FaultType::MultipleEchoMessages,
                });
            };
            return Ok(());
        }
        root_hash_state.insert_received_echo_message(sender_id.clone(), proof.to_owned());
        if !root_hash_state.is_ready_sent()
            && root_hash_state.count_received_echo_messages() >= min_guarantee_size
        {
            // it's high time to broadcast ready messages
            root_hash_state.turn_ready_sent_on();
            self.broadcast_ready_message(*root_hash, state.validators().clone())?;
        }
        if state.can_compute_output(root_hash) {
            self.compute_output(root_hash, state)?;
        }
        Ok(())
    }

    fn handle_ready(
        &self,
        sender_id: &Self::NodeId,
        message: ReadyMessage,
        state: &mut ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        let root_hash = message.as_ref();
        let max_durable_faulty_size = state.validator_set().max_durable_faulty_size();
        let root_hash_state = state.get_or_init_mut_root_hash_state(*root_hash);

        if !root_hash_state.insert_received_ready_message(sender_id.clone()) {
            self.handle_duplicated_ready_message(sender_id, root_hash);
            return Ok(());
        }

        if !root_hash_state.is_ready_sent()
            && root_hash_state.count_received_ready_messages() >= max_durable_faulty_size + 1
        {
            root_hash_state.turn_ready_sent_on();
            // to amplify ready messages, broadcast ready message to all
            self.broadcast_ready_message(*root_hash, state.validators().clone())?;
        }
        if state.can_compute_output(root_hash) {
            self.compute_output(root_hash, state)?;
        }
        Ok(())
    }

    fn compute_output(
        &self,
        root_hash: &Digest,
        state: &mut ReliableBroadcastState<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        let validators = state.validators();
        let root_hash_state = state.get_root_hash_state(root_hash);
        let mut shards: Vec<Option<Box<[u8]>>> = validators
            .keys()
            .map(|id| {
                root_hash_state
                    .get_received_echo_message(id)
                    .and_then(|proof| {
                        if proof.root_hash() == root_hash {
                            Some(proof.value().clone().into_boxed_slice())
                        } else {
                            None
                        }
                    })
            })
            .collect();
        let decoder = state.encoder();
        let output = decode_from_shards(&decoder, &mut shards, Some(root_hash))?;
        state.set_output(output);
        Ok(())
    }

    fn broadcast_echo_message(
        &self,
        value: Proof<Vec<u8>>,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _index) in validators {
            if node_id != *self.my_id() {
                let message = BroadcastMessage::Echo(value.clone().into());
                self.send_message(node_id, message);
            }
        }
        Ok(())
    }

    fn broadcast_ready_message(
        &self,
        value: Digest,
        validators: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
    ) -> Result<()> {
        for (node_id, _) in validators {
            if node_id != *self.my_id() {
                let message = BroadcastMessage::Ready(value.into());
                self.send_message(node_id, message);
            }
        }
        Ok(())
    }
}

/// Breaks the input value into shards of equal length and encodes them and some extra parity shards
fn encode_to_shards(encoder: &Coder, mut value: Vec<u8>) -> Result<Vec<Vec<u8>>> {
    let data_shard_count = encoder.data_shard_count();
    let parity_shard_count = encoder.parity_shard_count();

    // insert the length of `value` so it can be decoded without the padding.
    let payload_len = value.len() as u32;
    value.splice(0..0, 0..4); // Insert 4 bytes at the beginning.
    BigEndian::write_u32(&mut value[..4], payload_len); // Write the size.

    // Size of a Merkle tree leaf value: the value size divided by the number of data shards,
    // and rounded up, so that the full value always fits in the data shards. Always at least 1.
    let shard_len = (value.len() + data_shard_count - 1) / data_shard_count;
    // Pad the last data shard with zeros. Fill the parity shards with zeros.
    value.resize(shard_len * (data_shard_count + parity_shard_count), 0);

    let mut shards: Vec<&mut [u8]> = value.chunks_mut(shard_len).collect();

    // Construct the parity chunks/shards. This only fails if a shard is empty or the shards
    // have different sizes. Our shards all have size `shard_len`, which is at least 1.
    encoder.encode(&mut shards).expect("wrong shard size");

    Ok(shards.into_iter().map(|shard| shard.to_vec()).collect())
}

fn decode_from_shards(
    decoder: &Coder,
    shards: &mut [Option<Box<[u8]>>],
    root_hash: Option<&Digest>,
) -> Result<Vec<u8>> {
    // Try to interpolate the Merkle tree using the Reed-Solomon erasure coding scheme.
    decoder.reconstruct(shards)?;

    // Collect shards for tree construction.
    let shards: Vec<Vec<u8>> = shards
        .iter()
        .filter_map(|shard| shard.as_ref().map(|v| v.to_vec()))
        .collect();

    let merkle_tree = MerkleTree::from(shards);
    if let Some(root_hash_to_be_checked) = root_hash {
        if merkle_tree.root_hash() != root_hash_to_be_checked {
            return Err(Error::IllegalMerkleTreeRootHash);
        }
    }
    let count = decoder.data_shard_count();
    let mut bytes = merkle_tree.into_values().into_iter().take(count).flatten();
    let payload_len = match (bytes.next(), bytes.next(), bytes.next(), bytes.next()) {
        (Some(b0), Some(b1), Some(b2), Some(b3)) => {
            Ok(BigEndian::read_u32(&[b0, b1, b2, b3]) as usize)
        }
        _ => Err(Error::MerkleTreeMissingPayloadLength), // The proposer is faulty: no payload size.
    }?;
    let payload: Vec<u8> = bytes.take(payload_len).collect();
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode() {
        let data_shards = 3;
        let parity_shards = 2;
        let coder = Coder::new(data_shards, parity_shards).unwrap();
        assert_eq!(data_shards, coder.data_shard_count());
        assert_eq!(parity_shards, coder.parity_shard_count());

        let input = b"test";

        let shards = encode_to_shards(&coder, input.clone().to_vec()).unwrap();
        let mut shards: Vec<Option<Box<[u8]>>> = shards
            .into_iter()
            .map(|shard| Some(shard.into_boxed_slice()))
            .collect();
        assert_eq!(5, shards.len());
        shards[0] = None;
        shards[4] = None;

        let decoded = decode_from_shards(&coder, shards.as_mut_slice(), None).unwrap();
        assert_eq!(input, decoded.as_slice());
    }
}
