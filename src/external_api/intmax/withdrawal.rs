use alloy::primitives::{Bytes, TxHash, B256};
use log::info;
use mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr as _;

use crate::{
    external_api::{contracts::int1::Int1Contract, intmax::header::VersionHeader as _},
    utils::{
        config::Settings,
        network::{get_network, Network},
        retry::with_retry,
        time::sleep_for,
    },
};

use super::error::{IntmaxError, IntmaxErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitWithdrawalInput {
    pub public_inputs: SimpleWithdrawalPublicInputs,
    pub proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitWithdrawalSuccess {
    pub withdrawal_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum SubmitWithdrawalResponse {
    Success(SubmitWithdrawalSuccess),
    Error(IntmaxErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryWithdrawalSuccess {
    pub status: String,
    pub transaction_hash: Option<TxHash>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum QueryWithdrawalResponse {
    Success(QueryWithdrawalSuccess),
    Error(IntmaxErrorResponse),
}

async fn start_withdrawal(
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> Result<String, IntmaxError> {
    let settings = Settings::load().unwrap();
    let input = SubmitWithdrawalInput {
        public_inputs: pis,
        proof: "0x".to_string() + proof, // add 0x prefix
    };
    let url = format!("{}/submit-proof", settings.api.withdrawal_server_url);
    info!("Submitting withdrawal to {}, body: {:?}", url, input);
    let response = with_retry(|| async {
        reqwest::Client::new()
            .post(url.clone())
            .json(&input)
            .with_version_header()
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request withdrawal server".to_string()))?;
    let response: Value = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!("failed to parse response as json: {}", e))
    })?;
    let response: SubmitWithdrawalResponse =
        serde_json::from_value(response.clone()).map_err(|_| {
            IntmaxError::SerializeError(format!("failed to parse response: {}", response))
        })?;

    match response {
        SubmitWithdrawalResponse::Success(success) => Ok(success.withdrawal_id),
        SubmitWithdrawalResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

async fn query_withdrawal(withdrawal_id: &str) -> Result<QueryWithdrawalSuccess, IntmaxError> {
    let settings = Settings::load().unwrap();
    let url = format!(
        "{}/{}/proof-status",
        settings.api.withdrawal_server_url, withdrawal_id
    );
    let response = with_retry(|| async {
        reqwest::Client::new()
            .get(url.clone())
            .with_version_header()
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to query withdrawal server".to_string()))?;
    let response: Value = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!("failed to parse response as json: {}", e))
    })?;
    let response: QueryWithdrawalResponse =
        serde_json::from_value(response.clone()).map_err(|_| {
            IntmaxError::SerializeError(format!("failed to parse response: {}", response))
        })?;

    match response {
        QueryWithdrawalResponse::Success(success) => Ok(success),
        QueryWithdrawalResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

pub async fn submit_withdrawal(
    int1: &Int1Contract,
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> Result<TxHash, IntmaxError> {
    info!("submit_withdrawal with args {:?} proof {}", pis, proof);
    if get_network() == Network::Localnet {
        let local_private_key: B256 =
            "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
                .parse()
                .unwrap();
        let proof_hex = Bytes::from_str(proof).unwrap();
        let tx_hash = int1
            .withdrawal(local_private_key, &pis, proof_hex.to_vec())
            .await?;
        Ok(tx_hash)
    } else {
        let withdrawal_id = start_withdrawal(pis, proof).await?;
        let max_try = 5;
        let mut try_count = 0;
        let cooldown = 60;
        loop {
            try_count += 1;
            if try_count > max_try {
                return Err(IntmaxError::InternalError("withdrawal timeout".to_string()));
            }
            let status = query_withdrawal(&withdrawal_id).await?;
            match status.status.as_str() {
                "pending" => {
                    info!("withdrawal is pending");
                    sleep_for(cooldown);
                }
                "processing" => {
                    info!("withdrawal is processing");
                    sleep_for(cooldown);
                }
                "completed" => return Ok(status.transaction_hash.unwrap()),
                "failed" => {
                    return Err(IntmaxError::InternalError(format!(
                        "withdrawal failed: {}",
                        status.status
                    )));
                }
                "not_found" => {
                    return Err(IntmaxError::InternalError(format!(
                        "withdrawal not found: {}",
                        status.status
                    )));
                }
                _ => {
                    return Err(IntmaxError::InternalError(format!(
                        "unexpected status: {}",
                        status.status
                    )));
                }
            }
        }
    }
}
