use crate::external_api::contracts::error::BlockchainError;

#[derive(Debug, thiserror::Error)]
pub enum GraphClientError {
    #[error("Blockchain error: {0}")]
    BlockchainError(#[from] BlockchainError),
}
