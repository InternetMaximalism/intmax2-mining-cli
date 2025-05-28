use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::external_api::contracts::error::BlockchainError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntmaxErrorResponse {
    pub code: String,
    pub message: String,
    pub errors: Option<Value>,
}

#[derive(thiserror::Error, Debug)]
pub enum IntmaxError {
    #[error("Blockchain error: {0}")]
    BlockchainError(#[from] BlockchainError),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("Server error: {0:?}")]
    ServerError(IntmaxErrorResponse),
    #[error("Internal error: {0}")]
    InternalError(String),
}
