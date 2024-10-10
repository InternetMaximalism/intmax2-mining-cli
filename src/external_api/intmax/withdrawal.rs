use ethers::{
    providers::{Http, PendingTransaction},
    types::{Address, Bytes, H256, U256},
};
use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait;
use log::info;
use mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use crate::{
    external_api::contracts::int1::{get_int1_contract_with_signer, int_1},
    utils::{
        config::Settings,
        network::{get_network, Network},
        retry::with_retry,
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
enum SumbitWithdrawalResponse {
    Sucess(SubmitWithdrawalSuccess),
    Error(IntmaxErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryWithdrawalSuccess {
    pub status: String,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum QueryWithdrawalResponse {
    Sucess(QueryWithdrawalSuccess),
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
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request withdrawal server".to_string()))?;
    let response: Value = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!(
            "failed to parse response as json: {}",
            e.to_string()
        ))
    })?;
    let response: SumbitWithdrawalResponse =
        serde_json::from_value(response.clone()).map_err(|_| {
            IntmaxError::SerializeError(format!("failed to parse response: {}", response))
        })?;

    match response {
        SumbitWithdrawalResponse::Sucess(success) => Ok(success.withdrawal_id),
        SumbitWithdrawalResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

async fn query_withdrawal(withdrawal_id: &str) -> Result<QueryWithdrawalSuccess, IntmaxError> {
    let settings = Settings::load().unwrap();
    let url = format!(
        "{}/{}/proof-status",
        settings.api.withdrawal_server_url, withdrawal_id
    );
    let response = with_retry(|| async { reqwest::Client::new().get(url.clone()).send().await })
        .await
        .map_err(|_| IntmaxError::NetworkError("failed to query withdrawal server".to_string()))?;
    let response: Value = response.json().await.map_err(|e| {
        IntmaxError::SerializeError(format!(
            "failed to parse response as json: {}",
            e.to_string()
        ))
    })?;
    let response: QueryWithdrawalResponse =
        serde_json::from_value(response.clone()).map_err(|_| {
            IntmaxError::SerializeError(format!("failed to parse response: {}", response))
        })?;

    match response {
        QueryWithdrawalResponse::Sucess(success) => Ok(success),
        QueryWithdrawalResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

pub async fn submit_withdrawal(
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> Result<H256, IntmaxError> {
    info!("submit_withdrawal with args {:?} proof {}", pis, proof);
    let tx_hash = if get_network() == Network::Localnet {
        let tx_hash = localnet_withdrawal(pis, proof).await.unwrap();
        tx_hash
    } else {
        let withdrawal_id = start_withdrawal(pis, proof).await?;
        let max_try = 5;
        let mut try_count = 0;
        loop {
            try_count += 1;
            if try_count > max_try {
                return Err(IntmaxError::InternalError("withdrawal timeout".to_string()));
            }
            let status = query_withdrawal(&withdrawal_id).await?;
            match status.status.as_str() {
                "pending" => {
                    info!("withdrawal is pending");
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
                "processing" => {
                    info!("withdrawal is processing");
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                }
                "completed" => {
                    let tx_hash = H256::from_str(&status.transaction_hash.unwrap()).unwrap();
                    break tx_hash;
                }
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
    };
    Ok(tx_hash)
}

async fn localnet_withdrawal(
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> anyhow::Result<H256> {
    // Hardhat's default private key for withdrawal
    let local_private_key =
        H256::from_str("0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a")
            .unwrap();
    let int1 = get_int1_contract_with_signer(local_private_key).await?;
    let public_inputs = int_1::WithdrawalPublicInputs {
        deposit_root: pis.deposit_root.to_bytes_be().try_into().unwrap(),
        nullifier: pis.nullifier.to_bytes_be().try_into().unwrap(),
        recipient: Address::from_slice(&pis.recipient.to_bytes_be()),
        token_index: pis.token_index,
        amount: U256::from_big_endian(&pis.amount.to_bytes_be()),
    };
    let proof = Bytes::from_str(proof)?;
    let tx = int1.withdraw(public_inputs, proof);
    let pending_tx: PendingTransaction<Http> = match tx.send().await {
        Ok(tx) => {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            tx
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
        }
    };
    Ok(pending_tx.tx_hash())
}

#[cfg(test)]
mod tests {
    use intmax2_zkp::ethereum_types::{address::Address, bytes32::Bytes32, u256::U256};

    #[tokio::test]
    #[ignore]
    async fn test_submit_withdrawal() {
        let pis =
            mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs {
                deposit_root: Bytes32::default(),
                nullifier: Bytes32::default(),
                recipient: Address::default(),
                token_index: 0,
                amount: U256::default(),
            };
        let proof = "0x12345678";
        let tx_hash = super::submit_withdrawal(pis, proof).await.unwrap();
        println!("tx_hash: {:?}", tx_hash);
    }
}
