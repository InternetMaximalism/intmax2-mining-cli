use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    external_api::intmax::header::VersionHeader as _,
    utils::{config::Settings, retry::with_retry},
};

use super::error::{IntmaxError, IntmaxErrorResponse};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPriceSuccessResponse {
    pub max_fee_per_gas: u64,
    pub max_priority_fee_per_gas: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GasPriceResponse {
    Success(GasPriceSuccessResponse),
    Error(IntmaxErrorResponse),
}

pub async fn get_gas_estimation() -> Result<GasPriceSuccessResponse, IntmaxError> {
    info!("Getting gas price");
    let settings = Settings::load().unwrap();
    let response = with_retry(|| async {
        reqwest::Client::new()
            .get(settings.api.gas_server_url.clone())
            .with_version_header()
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request circulation server".to_string()))?;
    let response_json: GasPriceResponse = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!("failed to parse response: {}", e.to_string()))
    })?;
    match response_json {
        GasPriceResponse::Success(success) => Ok(success),
        GasPriceResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_gas_price() {
        dotenv::dotenv().ok();
        let response = get_gas_estimation().await.unwrap();
        dbg!(response);
    }
}
