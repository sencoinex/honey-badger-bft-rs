use asynchronous_common_subset::AsynchronousCommonSubset;
use binary_agreement::{
    epoch::Epoch,
    message::{BinaryAgreementMessage, BinaryAgreementMessageContent},
    node::NodeMessage as BaNodeMessage,
    BinaryAgreement,
};
use logger::prelude::*;
use rand::thread_rng;
use reliable_broadcast::{
    message::BroadcastMessage, node::NodeMessage as RbcNodeMessage, ReliableBroadcast,
};
use std::collections::{BTreeMap, VecDeque};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::{fmt, thread};
use threshold_crypto::{SecretKeyShare, SecretKeyShares};

type NodeId = u16;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
struct Index(u64);

impl From<u16> for Index {
    fn from(value: u16) -> Self {
        Self(value as u64)
    }
}

impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl Into<u64> for Index {
    fn into(self) -> u64 {
        self.0
    }
}

impl Into<usize> for Index {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl AsRef<u64> for Index {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

type SessionId = String;

struct ReliableBroadcastImpl {
    id: NodeId,
    target_id: NodeId,
    message_receiver: Arc<Mutex<Receiver<RbcNodeMessage<NodeId>>>>,
    message_router: BTreeMap<NodeId, SyncSender<RbcNodeMessage<NodeId>>>,
}

impl fmt::Debug for ReliableBroadcastImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.id, self.target_id)
    }
}

impl ReliableBroadcast for ReliableBroadcastImpl {
    type NodeId = NodeId;
    type ValidatorIndex = Index;

    fn my_id(&self) -> &NodeId {
        &self.id
    }

    fn next_message(&self) -> RbcNodeMessage<NodeId> {
        let receiver = self.message_receiver.lock().unwrap();
        let message = receiver.recv().unwrap();
        let sender_id = match &message {
            RbcNodeMessage::BroadcastMessage {
                sender_id,
                message: _,
            } => sender_id,
            RbcNodeMessage::Terminate => self.my_id(),
        };
        debug!("[receive message]{} -> {:?}", sender_id, self);
        message
    }

    fn send_message(&self, target_id: NodeId, message: BroadcastMessage) {
        assert_ne!(self.id, target_id);
        let message_type = match message {
            BroadcastMessage::Value(_) => "value message",
            BroadcastMessage::Echo(_) => "echo message",
            BroadcastMessage::Ready(_) => "ready message",
        };
        debug!(
            "[send message]{:?} -> {}-{}: {}",
            self, target_id, self.id, message_type
        );
        let sender = self.message_router.get(&target_id).unwrap();
        sender
            .send(RbcNodeMessage::BroadcastMessage {
                sender_id: self.id,
                message,
            })
            .expect("message should be sent without error...");
    }
}

struct BinaryAgreementImpl {
    id: NodeId,
    target_id: NodeId,
    message_receiver: Arc<Mutex<Receiver<BaNodeMessage<NodeId>>>>,
    message_router: BTreeMap<NodeId, SyncSender<BaNodeMessage<NodeId>>>,
    message_queue: BTreeMap<Epoch, VecDeque<BaNodeMessage<NodeId>>>,
}

impl fmt::Debug for BinaryAgreementImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.id, self.target_id)
    }
}

impl BinaryAgreement for BinaryAgreementImpl {
    type NodeId = NodeId;
    type ValidatorIndex = Index;
    type SessionId = SessionId;

    fn my_id(&self) -> &Self::NodeId {
        &self.id
    }

    fn next_message(&mut self, epoch: &Epoch) -> BaNodeMessage<NodeId> {
        if let Some(queue) = self.message_queue.get_mut(epoch) {
            if let Some(queued_message) = queue.pop_front() {
                return queued_message;
            }
        }
        loop {
            let message = self.message_receiver.lock().unwrap().recv().unwrap();
            match &message {
                BaNodeMessage::BinaryAgreementMessage {
                    sender_id,
                    message:
                        BinaryAgreementMessage {
                            epoch: message_epoch,
                            content: _,
                        },
                } => {
                    debug!("[receive message]{} -> {}", sender_id, self.id);
                    if message_epoch == epoch {
                        return message;
                    } else {
                        let queue = self
                            .message_queue
                            .entry(*message_epoch)
                            .or_insert_with(VecDeque::new);
                        queue.push_back(message);
                    }
                }
                BaNodeMessage::Terminate => {
                    return message;
                }
            };
        }
    }

    fn send_message(&self, target_id: Self::NodeId, message: BinaryAgreementMessage) {
        let message_type = match message.content {
            BinaryAgreementMessageContent::BVal(_) => "BVal",
            BinaryAgreementMessageContent::Aux(_) => "AUX",
            BinaryAgreementMessageContent::Conf(_) => "CONF",
            BinaryAgreementMessageContent::Coin(_) => "COIN",
        };
        debug!(
            "[send message]{} -> {}: {:?} {}",
            self.id, target_id, message.epoch, message_type
        );
        let sender = self.message_router.get(&target_id).unwrap();
        sender
            .send(BaNodeMessage::BinaryAgreementMessage {
                sender_id: self.id,
                message,
            })
            .expect("message should be sent without error...");
    }

    fn on_next_epoch(&mut self, epoch: &Epoch) {
        debug!("[begin epoch]{:?}: {:?}", self, epoch);
    }
}

struct TestNode {
    id: NodeId,
    index: Index,
    rbc_message_receivers: BTreeMap<NodeId, Arc<Mutex<Receiver<RbcNodeMessage<NodeId>>>>>,
    rbc_message_router: BTreeMap<NodeId, BTreeMap<NodeId, SyncSender<RbcNodeMessage<NodeId>>>>,
    ba_message_receivers: BTreeMap<NodeId, Arc<Mutex<Receiver<BaNodeMessage<NodeId>>>>>,
    ba_message_router: BTreeMap<NodeId, BTreeMap<NodeId, SyncSender<BaNodeMessage<NodeId>>>>,
}

impl fmt::Debug for TestNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'a> AsynchronousCommonSubset for TestNode {
    type NodeId = NodeId;
    type ValidatorIndex = Index;
    type SessionId = SessionId;
    type ReliableBroadcast = ReliableBroadcastImpl;
    type BinaryAgreement = BinaryAgreementImpl;

    fn my_id(&self) -> &Self::NodeId {
        &self.id
    }

    fn create_reliable_broadcast_instance(
        &mut self,
        target_id: &NodeId,
    ) -> Self::ReliableBroadcast {
        debug!("RBC instance created: {}-{}", self.id, target_id);
        let message_receiver = self.rbc_message_receivers.get(target_id).unwrap().clone();
        let mut message_router = BTreeMap::default();
        for (acs_node_id, senders) in &self.rbc_message_router {
            for (rbc_instance_id, sender) in senders {
                if rbc_instance_id == target_id {
                    message_router.insert(acs_node_id.clone(), sender.clone());
                }
            }
        }
        ReliableBroadcastImpl {
            id: self.id.clone(),
            target_id: target_id.clone(),
            message_receiver,
            message_router,
        }
    }

    fn terminate_reliable_broadcast(&self, target_id: &Self::NodeId) {
        debug!("let's terminate RBC instance: {}-{}", self.id, target_id);
        let sender = self
            .rbc_message_router
            .get(&self.id)
            .unwrap()
            .get(target_id)
            .unwrap();
        sender
            .send(RbcNodeMessage::Terminate)
            .expect("rbc terminate message should be sent without error...");
    }

    fn create_binary_agreement_instance(&mut self, target_id: &NodeId) -> Self::BinaryAgreement {
        debug!("BA instance created: {}-{}", self.id, target_id);
        let message_receiver = self.ba_message_receivers.get(target_id).unwrap().clone();
        let mut message_router = BTreeMap::default();
        for (acs_node_id, senders) in &self.ba_message_router {
            for (rbc_instance_id, sender) in senders {
                if rbc_instance_id == target_id {
                    message_router.insert(acs_node_id.clone(), sender.clone());
                }
            }
        }
        BinaryAgreementImpl {
            id: self.id.clone(),
            target_id: target_id.clone(),
            message_receiver,
            message_router,
            message_queue: BTreeMap::new(),
        }
    }

    fn get_binary_agreement_session_id(&self, target_id: &NodeId) -> Self::SessionId {
        format!("test-{}", target_id)
    }
}

fn gen_random_secret_key_shares(threshold: usize) -> SecretKeyShares {
    let mut rnd = thread_rng();
    SecretKeyShares::random(threshold, &mut rnd)
}

#[test]
fn test_simple_procedure() {
    // init logger
    let mut builder = logger::default::DefaultLoggerBuilder::new();
    builder.is_async(true);
    builder.level(logger::Level::Debug);
    let _logger = builder.build();

    let channel_size = 10000;
    // { acs_node_id: { target_index: (rbc_receiver, ba_receiver) } }
    let mut message_receivers: BTreeMap<
        NodeId,
        BTreeMap<
            NodeId,
            (
                Arc<Mutex<Receiver<RbcNodeMessage<NodeId>>>>,
                Arc<Mutex<Receiver<BaNodeMessage<NodeId>>>>,
            ),
        >,
    > = BTreeMap::new();
    // { acs_index: { send_target_node_id: sender } }
    let mut rbc_message_router: BTreeMap<
        NodeId,
        BTreeMap<NodeId, SyncSender<RbcNodeMessage<NodeId>>>,
    > = BTreeMap::new();
    // { acs_index: { send_target_node_id: sender } }
    let mut ba_message_router: BTreeMap<
        NodeId,
        BTreeMap<NodeId, SyncSender<BaNodeMessage<NodeId>>>,
    > = BTreeMap::new();
    for id in 1..=4 {
        message_receivers.insert(id, BTreeMap::default());
        rbc_message_router.insert(id, BTreeMap::default());
        ba_message_router.insert(id, BTreeMap::default());
        for child_id in 1..=4 {
            let (rb_sender, rb_receiver) = sync_channel(channel_size);
            let (ba_sender, ba_receiver) = sync_channel(channel_size);
            message_receivers.get_mut(&id).unwrap().insert(
                child_id,
                (
                    Arc::new(Mutex::new(rb_receiver)),
                    Arc::new(Mutex::new(ba_receiver)),
                ),
            );
            rbc_message_router
                .get_mut(&id)
                .unwrap()
                .insert(child_id, rb_sender);
            ba_message_router
                .get_mut(&id)
                .unwrap()
                .insert(child_id, ba_sender);
        }
    }
    let mut nodes: BTreeMap<NodeId, TestNode> = BTreeMap::new();
    for (id, receivers) in message_receivers {
        let index: Index = (id - 1).into();
        let mut rbc_message_receivers = BTreeMap::new();
        let mut ba_message_receivers = BTreeMap::new();
        for (child_index, (rbc_receiver, ba_receiver)) in receivers {
            rbc_message_receivers.insert(child_index, rbc_receiver);
            ba_message_receivers.insert(child_index, ba_receiver);
        }
        nodes.insert(
            id,
            TestNode {
                id,
                index,
                rbc_message_receivers,
                rbc_message_router: rbc_message_router.clone(),
                ba_message_receivers,
                ba_message_router: ba_message_router.clone(),
            },
        );
    }

    let inputs: Vec<&str> = vec!["Foo1", "Foo2", "Foo3", "Foo4"];
    let mut handles = BTreeMap::new();
    let validator_indices: BTreeMap<NodeId, Index> =
        nodes.iter().map(|(id, node)| (*id, node.index)).collect();
    let threshold = (nodes.len() - 1) / 3;
    let secret_key_shares = gen_random_secret_key_shares(threshold);
    let public_key_shares = secret_key_shares.public_keys();
    let secret_key_share_map: BTreeMap<NodeId, SecretKeyShare> = validator_indices
        .iter()
        .map(|(node_id, index)| {
            let secret_key_share = secret_key_shares.secret_key_share(*index.as_ref());
            (node_id.clone(), secret_key_share)
        })
        .collect();
    for (id, mut node) in nodes {
        let input = inputs.get((id - 1) as usize).unwrap().as_bytes().to_vec();
        let validator_indices = validator_indices.clone();
        let secret_key_share_map = secret_key_share_map.clone();
        let public_key_shares = public_key_shares.clone();
        let handle = thread::spawn(move || {
            let secret_key_share = secret_key_share_map.get(&id).unwrap().clone();
            node.propose(
                input,
                validator_indices,
                secret_key_share,
                public_key_shares,
            )
        });
        handles.insert(id, handle);
    }
    for (id, handle) in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(state) => {
                let outputs = state.into_output();
                let outputs: Vec<&str> = outputs
                    .iter()
                    .map(|(_id, bytes)| match bytes {
                        Some(bytes) => std::str::from_utf8(bytes).unwrap(),
                        None => "",
                    })
                    .collect();
                println!("acs {} got outputs: {:?}", id, outputs);
                assert_eq!(outputs, inputs.clone());
            }
            Err(err) => {
                panic!("id: {:?}, err: {:?}", id, err);
            }
        }
    }
}
