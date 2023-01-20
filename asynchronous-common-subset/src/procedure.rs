use crate::{
    node::NodeId, session::SessionId, validator::ValidatorIndex, AsynchronousCommonSubsetState,
    Result,
};
use binary_agreement::BinaryAgreement;
use core::fmt;
use reliable_broadcast::ReliableBroadcast;
use std::collections::{BTreeMap, HashMap};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use threshold_crypto::{PublicKeyShares, SecretKeyShare};

pub trait AsynchronousCommonSubset: fmt::Debug {
    type NodeId: NodeId + 'static;
    type ValidatorIndex: ValidatorIndex + 'static;
    type SessionId: SessionId + 'static;
    type ReliableBroadcast: ReliableBroadcast<NodeId = Self::NodeId, ValidatorIndex = Self::ValidatorIndex>
        + Send
        + 'static;
    type BinaryAgreement: BinaryAgreement<
            NodeId = Self::NodeId,
            ValidatorIndex = Self::ValidatorIndex,
            SessionId = Self::SessionId,
        > + Send
        + 'static;

    fn my_id(&self) -> &Self::NodeId;

    fn create_reliable_broadcast_instance(
        &mut self,
        target_id: &Self::NodeId,
    ) -> Self::ReliableBroadcast;

    fn terminate_reliable_broadcast(&self, target_id: &Self::NodeId);

    fn create_binary_agreement_instance(
        &mut self,
        target_id: &Self::NodeId,
    ) -> Self::BinaryAgreement;

    fn get_binary_agreement_session_id(&self, target_id: &Self::NodeId) -> Self::SessionId;

    /// Let {RBCi}N refer to N instances of the reliable broadcast protocol, where Pi is the sender of RBCi.
    /// Let {BAi}N refer to N instances of the binary byzantine agreement protocol.
    /// * upon receiving input vi, input vi to RBCi
    /// * upon delivery of vj from RBCj, if input has not yet been provided to BAj, then provide input 1 to BAj.
    /// * upon delivery of value 1 from at least N − f instances of BA, provide input 0 to each instance of BA that has not yet been provided input.
    /// * once all instances of BA have completed, let C ⊂ [1..N] be the
    /// indexes of each BA that delivered 1. Wait for the output v j for eachRBCj such that j∈C.Finally output ∪j∈Cvj.
    fn propose(
        &mut self,
        input: Vec<u8>,
        validator_indices: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
        secret_key_share: SecretKeyShare,
        public_key_shares: PublicKeyShares,
    ) -> Result<AsynchronousCommonSubsetState<Self::NodeId>> {
        // initialize state
        let state: Arc<Mutex<AsynchronousCommonSubsetState<Self::NodeId>>> =
            Arc::new(Mutex::new(AsynchronousCommonSubsetState::new()));

        let rb_validator_set =
            reliable_broadcast::validator::ValidatorSet::new(validator_indices.clone())?;
        let ba_validator_set =
            binary_agreement::validator::ValidatorSet::new(validator_indices.clone())?;

        // create N instances of RBC and start their procedure
        let mut rb_threads = BTreeMap::new();
        let mut ba_receive_channels = HashMap::new();
        let mut ba_send_channels = HashMap::new();
        for (node_id, _validator_index) in validator_indices {
            let (ba_input_sender, ba_input_receiver) = mpsc::channel();
            ba_receive_channels.insert(node_id.clone(), ba_input_receiver);
            ba_send_channels.insert(node_id.clone(), ba_input_sender.clone());
            let rb_instance = self.create_reliable_broadcast_instance(&node_id);
            let validator_set = rb_validator_set.clone();
            let state_for_rb_thread = state.clone();
            let node_id_for_rb_thread = node_id.clone();
            let rb_thread = if &node_id == self.my_id() {
                let input = input.clone();
                thread::spawn(move || {
                    let rbc_out = rb_instance.propose(input, validator_set);
                    let ba_input = match &rbc_out {
                        Ok(rbc_out) => {
                            if rbc_out.is_decided() {
                                Some(true)
                            } else {
                                None
                            }
                        }
                        Err(_) => {
                            // TODO set fault logs (RBC failed)
                            None
                        }
                    };
                    let mut locked_state = state_for_rb_thread
                        .lock()
                        .expect("state mutex cannot be locked...");
                    locked_state
                        .set_binary_agreement_input(node_id_for_rb_thread.clone(), ba_input);
                    if let Ok(rbc_out) = rbc_out {
                        locked_state.set_reliable_broadcast_state(node_id_for_rb_thread, rbc_out);
                    }
                    ba_input_sender
                        .send(ba_input)
                        .expect("could not send binary agreement input message...");
                })
            } else {
                thread::spawn(move || {
                    let rbc_out = rb_instance.execute(None, validator_set);
                    let ba_input = match &rbc_out {
                        Ok(rbc_out) => {
                            if rbc_out.is_decided() {
                                Some(true)
                            } else {
                                None
                            }
                        }
                        Err(_) => {
                            // TODO set fault logs (RBC failed)
                            None
                        }
                    };
                    let mut locked_state = state_for_rb_thread
                        .lock()
                        .expect("state mutex cannot be locked...");
                    locked_state
                        .set_binary_agreement_input(node_id_for_rb_thread.clone(), ba_input);
                    if let Ok(rbc_out) = rbc_out {
                        locked_state.set_reliable_broadcast_state(node_id_for_rb_thread, rbc_out);
                    }
                    ba_input_sender
                        .send(ba_input)
                        .expect("could not send binary agreement input message...");
                })
            };
            rb_threads.insert(node_id, rb_thread);
        }

        // create N instances of BA
        let mut ba_threads = BTreeMap::new();
        let validator_key_shares = binary_agreement::validator::ValidatorKeyShares::new(
            secret_key_share,
            public_key_shares,
        );
        for (node_id, ba_input_receiver) in ba_receive_channels {
            let mut ba_instance = self.create_binary_agreement_instance(&node_id);
            let session_id = self.get_binary_agreement_session_id(&node_id);
            let validator_set = ba_validator_set.clone();
            let validator_key_shares = validator_key_shares.clone();
            let state_for_ba_thread = state.clone();
            let node_id_for_ba_thread = node_id.clone();
            let ba_send_channels = ba_send_channels.clone();
            let ba_thread = thread::spawn(move || {
                // get the first binary agreement input message and execute binary agreement procedure(let's ignore second and subsequent messages)
                let input = ba_input_receiver.recv().unwrap();
                if let Some(input) = input {
                    let ba_out = ba_instance.propose(
                        input,
                        validator_set.clone(),
                        validator_key_shares,
                        session_id,
                    );
                    let mut locked_state = state_for_ba_thread
                        .lock()
                        .expect("state mutex cannot be locked...");

                    if let Ok(ba_out) = ba_out {
                        locked_state.set_binary_agreement_state(node_id_for_ba_thread, ba_out);
                    } else {
                        // TODO set fault logs (ABA failed)
                    }
                    // if sum(aba_values) >= N - f then input false to binary agreement instance that has not started yet.
                    if locked_state.sum_binary_agreement_output()
                        >= validator_set.min_guarantee_size()
                    {
                        for (nid, _) in validator_set.as_indices() {
                            if !locked_state.has_binary_agreement_input(nid) {
                                let ba_input = Some(false);
                                locked_state.set_binary_agreement_input(nid.clone(), ba_input);
                                let ba_input_sender = ba_send_channels.get(nid).unwrap();
                                ba_input_sender
                                    .send(ba_input)
                                    .expect("could not send binary agreement input message...");
                            }
                        }
                    }
                }
            });
            ba_threads.insert(node_id, ba_thread);
        }

        // wait for all N BA instances to complete
        for (_node_id, ba_thread) in ba_threads {
            let _ = ba_thread.join().unwrap();
        }
        // terminate unfinished RB process
        {
            let locked_state = state.lock().expect("state mutex cannot be locked...");
            for (node_id, ba_out) in locked_state.as_binary_agreement_outputs() {
                if !ba_out.unwrap_or(false) {
                    self.terminate_reliable_broadcast(node_id);
                }
            }
        }
        for (_node_id, rb_thread) in rb_threads {
            let _ = rb_thread.join().unwrap();
        }

        let lock = Arc::try_unwrap(state).expect("state lock still has multiple owners...");
        let state = lock.into_inner().expect("state mutex cannot be locked...");
        Ok(state)
    }
}
