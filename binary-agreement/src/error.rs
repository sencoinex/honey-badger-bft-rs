use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid Binary Values: {value}")]
    InvalidBinaryValues { value: u8 },

    #[error("Error serializing session ID for coin: {session_id}")]
    SerializeCoinNameError { session_id: String },

    #[error("Threshold Encryption Error: {cause:?}")]
    ThresholdCryptError { cause: threshold_crypto::Error },

    #[error("Invalid combined signature hash")]
    InvalidCombinedSignature,
}

impl From<threshold_crypto::Error> for Error {
    fn from(cause: threshold_crypto::Error) -> Self {
        Self::ThresholdCryptError { cause }
    }
}
