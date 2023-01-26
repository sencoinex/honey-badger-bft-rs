use crate::{BatchTransactions, Epoch, Error, NodeId, Result, Transaction, ValidatorIndex};
use asynchronous_common_subset::AsynchronousCommonSubset;
use core::fmt;
use rand::Rng;
use std::collections::BTreeMap;
use threshold_crypto::{PublicKeyShares, SecretKeyShare};

pub trait HoneyBadger: fmt::Debug {
    type NodeId: NodeId + 'static;
    type ValidatorIndex: ValidatorIndex + 'static;
    type Transaction: Transaction;
    type BatchTransactions: BatchTransactions<Transaction = Self::Transaction>;
    type AsynchronousCommonSubset: AsynchronousCommonSubset<
        NodeId = Self::NodeId,
        ValidatorIndex = Self::ValidatorIndex,
    >;
    type Rng: Rng;
    fn rng(&mut self) -> &mut Self::Rng;

    fn create_asynchronous_common_subset_instance(
        &mut self,
        epoch: &Epoch,
    ) -> Self::AsynchronousCommonSubset;

    fn propose(
        &mut self,
        epoch: &Epoch,
        transactions: Self::BatchTransactions,
        validator_indices: BTreeMap<Self::NodeId, Self::ValidatorIndex>,
        secret_key_share: SecretKeyShare,
        public_key_shares: PublicKeyShares,
    ) -> Result<()> {
        let contribution_bytes = transactions
            .serialize()
            .map_err(|_| Error::BatchTransactionsSerializationError)?;
        let rng = self.rng();
        let ciphertext = public_key_shares
            .public_key()
            .encrypt_with_rng(rng, contribution_bytes);
        let encrypted_contribution_bytes = bincode::serialize(&ciphertext)
            .map_err(|err| Error::EncryptedBatchTransactionsSerializationError { cause: *err })?;
        let mut acs = self.create_asynchronous_common_subset_instance(epoch);
        let acs_result = acs.propose(
            encrypted_contribution_bytes,
            validator_indices,
            secret_key_share,
            public_key_shares,
        )?;
        // TODO for each acs output, broadcast its decryption share
        // TODO wait for f + 1 decryption share messages from other node
        // TODO decode asc outputs by received decryption share messages
        // TODO combine & sort transactions outputted by the process above -> block!
        todo!()
    }
}
