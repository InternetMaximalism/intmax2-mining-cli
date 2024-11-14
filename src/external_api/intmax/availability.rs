use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    external_api::intmax::header::VersionHeader as _,
    utils::{config::Settings, retry::with_retry},
};

use super::error::{IntmaxError, IntmaxErrorResponse};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvaliabilityServerSuccessResponse {
    pub is_available: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AvaliabilityServerResponse {
    Success(AvaliabilityServerSuccessResponse),
    Error(IntmaxErrorResponse),
}

pub async fn get_availability() -> Result<AvaliabilityServerSuccessResponse, IntmaxError> {
    info!("get_availability");
    let version = env!("CARGO_PKG_VERSION");
    let settings = Settings::load().unwrap();
    let response = with_retry(|| async {
        reqwest::Client::new()
            .get(format!(
                "{}?version={}",
                settings.api.availability_server_url, version,
            ))
            .with_version_header()
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request availability server".to_string()))?;
    let response_json: AvaliabilityServerResponse = response
        .json()
        .await
        .map_err(|e| IntmaxError::SerializeError(e.to_string()))?;
    match response_json {
        AvaliabilityServerResponse::Success(success) => Ok(success),
        AvaliabilityServerResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_availability() {
        let response = get_availability().await.unwrap();
        dbg!(response);
    }
}
