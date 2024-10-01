use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntmaxErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum IntmaxError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializeError(String),
    #[error("Server error: {0:?}")]
    ServerError(IntmaxErrorResponse),
    #[error("Internal error: {0}")]
    InternalError(String),
}
