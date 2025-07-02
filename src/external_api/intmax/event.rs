use alloy::{
    consensus::Transaction as _,
    primitives::{Address, TxHash},
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u256::U256};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    external_api::{
        contracts::{
            events::Deposited,
            utils::{get_batch_transaction, NormalProvider},
        },
        intmax::{
            error::{IntmaxError, IntmaxErrorResponse},
            header::VersionHeader as _,
        },
    },
    utils::{config::Settings, retry::with_retry},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositedEntry {
    pub deposit_id: u64,
    pub sender: Address,
    pub recipient_salt_hash: Bytes32,
    pub token_index: u32,
    pub amount: U256,
    pub transaction_hash: TxHash,
    pub block_timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum EventServerResponse {
    Success(Vec<DepositedEntry>),
    Error(IntmaxErrorResponse),
}

pub async fn get_deposit_events(
    provider: &NormalProvider,
    sender: Address,
) -> Result<Vec<Deposited>, IntmaxError> {
    info!("get_availability");
    let settings = Settings::load().unwrap();
    let response = with_retry(|| async {
        reqwest::Client::new()
            .get(format!(
                "{}/address/{}",
                settings.api.event_server_url,
                sender.to_string()
            ))
            .with_version_header()
            .send()
            .await
    })
    .await
    .map_err(|_| IntmaxError::NetworkError("failed to request availability server".to_string()))?;
    let response_json: EventServerResponse = response
        .json()
        .await
        .map_err(|e| IntmaxError::SerializeError(e.to_string()))?;

    match response_json {
        EventServerResponse::Success(events) => {
            let tx_hashes = events
                .iter()
                .map(|entry| entry.transaction_hash)
                .collect::<Vec<TxHash>>();
            let txs = get_batch_transaction(provider, &tx_hashes).await?;
            let mut deposited = Vec::new();
            for (event, tx) in events.iter().zip(txs.iter()) {
                deposited.push(Deposited {
                    sender: event.sender,
                    token_index: event.token_index,
                    deposit_id: event.deposit_id,
                    recipient_salt_hash: event.recipient_salt_hash,
                    amount: event.amount,
                    tx_nonce: tx.nonce(),
                    timestamp: event.block_timestamp,
                });
            }
            Ok(deposited)
        }
        EventServerResponse::Error(error) => Err(IntmaxError::ServerError(error)),
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::external_api::contracts::utils::get_provider;

    use super::*;

    #[tokio::test]
    async fn test_get_deposit_events() {
        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
        let address = env::var("DEPOSIT_ADDRESS")
            .map(|s| s.parse().unwrap())
            .unwrap_or_default();
        let provider = get_provider(&rpc_url).unwrap();
        let result = get_deposit_events(&provider, address).await;
        assert!(result.is_ok());
    }
}
