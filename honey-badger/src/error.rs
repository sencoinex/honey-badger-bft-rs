use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("ReliableBroadcastError: {cause}")]
    ReliableBroadcastError { cause: reliable_broadcast::Error },
    #[error("BinaryAgreementError: {cause}")]
    BinaryAgreementError { cause: binary_agreement::Error },
    #[error("BatchTransactionsSerializationError")]
    BatchTransactionsSerializationError,
    #[error("BatchTransactionsSerializationError: {cause:?}")]
    EncryptedBatchTransactionsSerializationError { cause: bincode::ErrorKind },
}

impl From<reliable_broadcast::Error> for Error {
    fn from(cause: reliable_broadcast::Error) -> Self {
        Self::ReliableBroadcastError { cause }
    }
}

impl From<binary_agreement::Error> for Error {
    fn from(cause: binary_agreement::Error) -> Self {
        Self::BinaryAgreementError { cause }
    }
}

impl From<asynchronous_common_subset::Error> for Error {
    fn from(value: asynchronous_common_subset::Error) -> Self {
        match value {
            asynchronous_common_subset::Error::ReliableBroadcastError { cause } => {
                Self::ReliableBroadcastError { cause }
            }
            asynchronous_common_subset::Error::BinaryAgreementError { cause } => {
                Self::BinaryAgreementError { cause }
            }
        }
    }
}
