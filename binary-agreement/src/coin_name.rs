use crate::{epoch::Epoch, session::SessionId, Error, Result};

pub struct CoinName(Vec<u8>);

impl CoinName {
    pub fn new<SID: SessionId>(session_id: &SID, epoch: &Epoch) -> Result<Self> {
        let name = bincode::serialize(&(session_id.to_string(), epoch.to_u64())).map_err(|_| {
            Error::SerializeCoinNameError {
                session_id: session_id.to_string(),
            }
        })?;
        Ok(Self(name))
    }
}

impl AsRef<[u8]> for CoinName {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}
