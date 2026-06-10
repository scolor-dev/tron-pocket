use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid mnemonic phrase")]
    InvalidMnemonic,

    #[error("invalid private key")]
    InvalidPrivateKey,

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error("incorrect password")]
    IncorrectPassword,

    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("tron node returned an error: {0}")]
    Node(String),

    #[error("(de)serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("encoding error: {0}")]
    Encoding(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
