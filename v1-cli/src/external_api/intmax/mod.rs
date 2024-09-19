use serde::{Deserialize, Serialize};

pub mod availability;
pub mod gnark;
pub mod withdrawal;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntmaxErrorResponse {
    pub code: String,
    pub message: String,
}
