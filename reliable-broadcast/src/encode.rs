use crate::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;

#[derive(Debug, Clone)]
pub struct Coder(Box<ReedSolomon>);

impl Coder {
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self> {
        let reed_solomon = ReedSolomon::new(data_shards, parity_shards)?;
        Ok(Self(Box::new(reed_solomon)))
    }

    /// Returns the number of data shards.
    pub fn data_shard_count(&self) -> usize {
        self.0.data_shard_count()
    }

    /// Returns the number of parity shards.
    pub fn parity_shard_count(&self) -> usize {
        self.0.parity_shard_count()
    }

    /// Constructs (and overwrites) the parity shards.
    pub fn encode(&self, shards: &mut [&mut [u8]]) -> Result<()> {
        Ok(self.0.encode(shards)?)
    }

    pub fn reconstruct(&self, shards: &mut [Option<Box<[u8]>>]) -> Result<()> {
        Ok(self.0.reconstruct(shards)?)
    }
}
