use serde::{Deserialize, Serialize};

use crate::config::Settings;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvaliabilityServerResponse {
    pub is_available: bool,
    pub message: String,
}

pub async fn get_availability() -> anyhow::Result<AvaliabilityServerResponse> {
    let settings = Settings::new()?;
    let response = reqwest::get(settings.api.availability_server_url).await?;
    let response_json: AvaliabilityServerResponse = response.json().await?;
    Ok(response_json)
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
