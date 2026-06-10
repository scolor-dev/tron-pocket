use serde::Serialize;

/// Error type returned to the frontend. Tauri serializes the `Err` variant
/// of a command's `Result` as the rejection value of the JS promise.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    InvalidMnemonic,
    InvalidPrivateKey,
    InvalidAddress(String),
    IncorrectPassword,
    NoWallet,
    AlreadyExists,
    Locked,
    Network(String),
    Node(String),
    Storage(String),
    Other(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AppError {}

impl From<tron_core::CoreError> for AppError {
    fn from(err: tron_core::CoreError) -> Self {
        use tron_core::CoreError;
        match err {
            CoreError::InvalidMnemonic => AppError::InvalidMnemonic,
            CoreError::InvalidPrivateKey => AppError::InvalidPrivateKey,
            CoreError::InvalidAddress(s) => AppError::InvalidAddress(s),
            CoreError::IncorrectPassword => AppError::IncorrectPassword,
            CoreError::Network(e) => AppError::Network(e.to_string()),
            CoreError::Node(s) => AppError::Node(s),
            CoreError::Serde(e) => AppError::Other(e.to_string()),
            CoreError::Encoding(s) => AppError::Other(s),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
