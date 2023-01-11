use crate::merkle::{Digest, Proof};

#[derive(Clone, PartialEq)]
pub enum BroadcastMessage {
    Value(ValueMessage),
    Echo(EchoMessage),
    Ready(ReadyMessage),
}

impl BroadcastMessage {
    pub fn into_value(self) -> Option<ValueMessage> {
        match self {
            Self::Value(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ValueMessage(Proof<Vec<u8>>);

impl ValueMessage {
    pub fn into_inner(self) -> Proof<Vec<u8>> {
        self.0
    }
}

impl AsRef<Proof<Vec<u8>>> for ValueMessage {
    fn as_ref(&self) -> &Proof<Vec<u8>> {
        &self.0
    }
}

impl From<Proof<Vec<u8>>> for ValueMessage {
    fn from(value: Proof<Vec<u8>>) -> Self {
        Self(value)
    }
}

#[derive(Clone, PartialEq)]
pub struct EchoMessage(Proof<Vec<u8>>);

impl EchoMessage {
    pub fn into_inner(self) -> Proof<Vec<u8>> {
        self.0
    }
}

impl AsRef<Proof<Vec<u8>>> for EchoMessage {
    fn as_ref(&self) -> &Proof<Vec<u8>> {
        &self.0
    }
}

impl From<Proof<Vec<u8>>> for EchoMessage {
    fn from(value: Proof<Vec<u8>>) -> Self {
        Self(value)
    }
}

#[derive(Clone, PartialEq)]
pub struct ReadyMessage(Digest);

impl ReadyMessage {
    pub fn into_inner(self) -> Digest {
        self.0
    }
}

impl AsRef<Digest> for ReadyMessage {
    fn as_ref(&self) -> &Digest {
        &self.0
    }
}

impl From<Digest> for ReadyMessage {
    fn from(value: Digest) -> Self {
        Self(value)
    }
}
