use crate::external_api::{contracts::error::BlockchainError, query::RequestError};

#[derive(Debug, thiserror::Error)]
pub enum GraphClientError {
    #[error("Blockchain error: {0}")]
    BlockchainError(#[from] BlockchainError),

    #[error("Request error: {0}")]
    RequestError(#[from] RequestError),

    #[error("Health check error: {0}")]
    HealthCheckError(String),
}
