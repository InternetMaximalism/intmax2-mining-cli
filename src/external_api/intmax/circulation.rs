use ethers::types::Address;
use log::info;
use serde::{Deserialize, Serialize};

use crate::utils::{config::Settings, errors::CLIError};

use super::IntmaxErrorResponse;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CirculationSuccessResponse {
    pub is_excluded: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CirculationResponse {
    Success(CirculationSuccessResponse),
    Error(IntmaxErrorResponse),
}

pub async fn get_circulation(address: Address) -> anyhow::Result<CirculationSuccessResponse> {
    info!("Getting circulation for address {:?}", address);
    let settings = Settings::new()?;
    let response = reqwest::get(format!(
        "{}/addresses/{:?}/exclusion",
        settings.api.circulation_server_url, address,
    ))
    .await
    .map_err(|e| CLIError::NetworkError(e.to_string()))?;
    let response_json: CirculationResponse = response.json().await?;
    match response_json {
        CirculationResponse::Success(success) => Ok(success),
        CirculationResponse::Error(error) => {
            anyhow::bail!("Circulation server error: {:?}", error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_circulation() {
        let address: Address = "0xFa1A4998136377DB9b09e24567bd6D17Ad78AaE6"
            .parse()
            .unwrap();
        let response = get_circulation(address).await.unwrap();
        dbg!(response);
    }
}
