use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Env error: {0}")]
    EnvError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Tx not found: {0}")]
    TxNotFound(String),
}
