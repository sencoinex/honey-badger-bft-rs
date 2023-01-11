use binary_agreement::epoch::Epoch;
use binary_agreement::message::{BinaryAgreementMessage, BinaryAgreementMessageContent};
use binary_agreement::node::NodeMessage;
use binary_agreement::validator::{ValidatorKeyShares, ValidatorSet};
use binary_agreement::BinaryAgreement;
use logger::prelude::*;
use rand::thread_rng;
use std::collections::{BTreeMap, VecDeque};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
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

type SessionId = u8;

struct TestNode {
    id: NodeId,
    index: Index,
    message_receiver: Receiver<NodeMessage<NodeId>>,
    message_router: BTreeMap<NodeId, SyncSender<NodeMessage<NodeId>>>,
    message_queue: BTreeMap<Epoch, VecDeque<NodeMessage<NodeId>>>,
}

impl fmt::Debug for TestNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl BinaryAgreement for TestNode {
    type NodeId = NodeId;
    type ValidatorIndex = Index;
    type SessionId = SessionId;

    fn my_id(&self) -> &Self::NodeId {
        &self.id
    }

    fn next_message(&mut self, epoch: &Epoch) -> NodeMessage<Self::NodeId> {
        if let Some(queue) = self.message_queue.get_mut(epoch) {
            if let Some(queued_message) = queue.pop_front() {
                return queued_message;
            }
        }
        loop {
            let message = self.message_receiver.recv().unwrap();
            match &message {
                NodeMessage::BinaryAgreementMessage {
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
                NodeMessage::Terminate => {
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
            .send(NodeMessage::BinaryAgreementMessage {
                sender_id: self.id,
                message,
            })
            .expect("message should be sent without error...");
    }

    fn on_next_epoch(&mut self, epoch: &Epoch) {
        debug!("[begin epoch]{}: {:?}", self.id, epoch);
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
    let mut message_receivers: BTreeMap<NodeId, Receiver<NodeMessage<NodeId>>> = BTreeMap::new();
    let mut message_router: BTreeMap<NodeId, SyncSender<NodeMessage<NodeId>>> = BTreeMap::new();
    for id in 1..=4 {
        let (sender, receiver) = sync_channel(channel_size);
        message_receivers.insert(id, receiver);
        message_router.insert(id, sender);
    }
    let mut nodes: BTreeMap<NodeId, TestNode> = BTreeMap::new();
    for (id, message_receiver) in message_receivers {
        nodes.insert(
            id,
            TestNode {
                id,
                index: (id - 1).into(),
                message_receiver,
                message_router: message_router.clone(),
                message_queue: BTreeMap::new(),
            },
        );
    }

    let session_id: SessionId = 1;
    let validator_indices = nodes.iter().map(|(id, node)| (*id, node.index)).collect();
    let validator_set = ValidatorSet::new(validator_indices).unwrap();
    let secret_key_shares = gen_random_secret_key_shares(validator_set.max_durable_faulty_size());
    let public_key_shares = secret_key_shares.public_keys();
    let secret_key_share_map: BTreeMap<NodeId, SecretKeyShare> = validator_set
        .as_indices()
        .iter()
        .map(|(node_id, index)| {
            let secret_key_share = secret_key_shares.secret_key_share(*index.as_ref());
            (node_id.clone(), secret_key_share)
        })
        .collect();
    let mut handles = BTreeMap::new();
    for (id, mut node) in nodes {
        let validator_set = validator_set.clone();
        let secret_key_share_map = secret_key_share_map.clone();
        let public_key_shares = public_key_shares.clone();
        let handle = thread::spawn(move || {
            let input = true;
            let secret_key_share = secret_key_share_map.get(&id).unwrap().clone();
            let validator_key_shares = ValidatorKeyShares::new(secret_key_share, public_key_shares);
            node.propose(input, validator_set, validator_key_shares, session_id)
        });
        handles.insert(id, handle);
    }
    for (id, handle) in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(state) => {
                assert!(state.is_decided());
                let output = state.get_output().unwrap();
                assert!(output);
            }
            Err(err) => {
                panic!("id: {:?}, err: {:?}", id, err);
            }
        }
    }
}
