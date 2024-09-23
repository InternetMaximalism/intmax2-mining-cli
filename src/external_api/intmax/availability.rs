use log::info;
use serde::{Deserialize, Serialize};

use crate::utils::{config::Settings, errors::CLIError};

use super::IntmaxErrorResponse;

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

pub async fn get_availability() -> anyhow::Result<AvaliabilityServerSuccessResponse> {
    info!("Getting availability");
    let version = env!("CARGO_PKG_VERSION");
    let settings = Settings::new()?;
    let response = reqwest::get(format!(
        "{}?version={}",
        settings.api.availability_server_url, version,
    ))
    .await
    .map_err(|e| CLIError::NetworkError(e.to_string()))?;
    let response_json: AvaliabilityServerResponse = response.json().await?;
    match response_json {
        AvaliabilityServerResponse::Success(success) => Ok(success),
        AvaliabilityServerResponse::Error(error) => {
            anyhow::bail!("Availability server error: {:?}", error)
        }
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
