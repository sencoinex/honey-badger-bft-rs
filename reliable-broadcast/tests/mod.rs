use logger::prelude::*;
use reliable_broadcast::message::BroadcastMessage;
use reliable_broadcast::node::NodeMessage;
use reliable_broadcast::validator::ValidatorSet;
use reliable_broadcast::ReliableBroadcast;
use std::collections::BTreeMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::{fmt, thread};

type Id = u16;

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

impl Into<usize> for Index {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct TestNode {
    id: Id,
    index: Index,
    message_receiver: Receiver<NodeMessage<Id>>,
    message_router: BTreeMap<Id, SyncSender<NodeMessage<Id>>>,
}

impl fmt::Debug for TestNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl ReliableBroadcast for TestNode {
    type NodeId = Id;
    type ValidatorIndex = Index;

    fn my_id(&self) -> &Id {
        &self.id
    }

    fn next_message(&self) -> NodeMessage<Id> {
        let message = self.message_receiver.recv().unwrap();
        let sender_id = match &message {
            NodeMessage::BroadcastMessage {
                sender_id,
                message: _,
            } => sender_id,
            NodeMessage::Terminate => self.my_id(),
        };
        debug!("[receive message]{} -> {}", sender_id, self.id);
        message
    }

    fn send_message(&self, target_id: Id, message: BroadcastMessage) {
        let message_type = match message {
            BroadcastMessage::Value(_) => "value message",
            BroadcastMessage::Echo(_) => "echo message",
            BroadcastMessage::Ready(_) => "ready message",
        };
        debug!(
            "[send message]{} -> {}: {}",
            self.id, target_id, message_type
        );
        let sender = self.message_router.get(&target_id).unwrap();
        sender
            .send(NodeMessage::BroadcastMessage {
                sender_id: self.id,
                message,
            })
            .expect("message should be sent without error...");
    }
}

#[test]
fn test_simple_procedure() {
    // init logger
    let mut builder = logger::default::DefaultLoggerBuilder::new();
    builder.is_async(true);
    builder.level(logger::Level::Debug);
    let _logger = builder.build();

    let channel_size = 10000;
    let mut message_receivers: BTreeMap<Id, Receiver<NodeMessage<Id>>> = BTreeMap::new();
    let mut message_router: BTreeMap<Id, SyncSender<NodeMessage<Id>>> = BTreeMap::new();
    for id in 1..=4 {
        let (sender, receiver) = sync_channel(channel_size);
        message_receivers.insert(id, receiver);
        message_router.insert(id, sender);
    }
    let mut nodes: BTreeMap<Id, TestNode> = BTreeMap::new();
    for (id, message_receiver) in message_receivers {
        nodes.insert(
            id,
            TestNode {
                id,
                index: (id - 1).into(),
                message_receiver,
                message_router: message_router.clone(),
            },
        );
    }
    let input = b"Foo";
    let proposer_id = 1;
    let validator_indices = nodes.iter().map(|(id, node)| (*id, node.index)).collect();
    let validator_set = ValidatorSet::new(validator_indices).unwrap();
    let mut handles = BTreeMap::new();
    for (id, node) in nodes {
        let validator_set = validator_set.clone();
        let handle = if id == proposer_id {
            thread::spawn(move || node.propose(input.to_vec(), validator_set))
        } else {
            thread::spawn(move || node.execute(None, validator_set))
        };
        handles.insert(id, handle);
    }
    for (id, handle) in handles {
        let result = handle.join().unwrap();
        match result {
            Ok(state) => {
                assert!(state.is_decided());
                let output = state.get_output().unwrap().as_slice();
                assert_eq!(input, output);
            }
            Err(err) => {
                panic!("id: {:?}, err: {:?}", id, err);
            }
        }
    }
}
