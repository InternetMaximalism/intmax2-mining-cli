use alloy::consensus::Transaction;
use alloy::primitives::{Address, TxHash};
use intmax2_zkp::ethereum_types::bytes32::Bytes32;
use intmax2_zkp::ethereum_types::u256::U256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::external_api::contracts::utils::get_batch_transaction;
use crate::external_api::{
    contracts::{
        events::{DepositLeafInserted, Deposited},
        utils::NormalProvider,
    },
    query::post_request_with_bearer_token,
};

use super::error::GraphClientError;

// A wrapper around TheGraphClient that provides additional functionality for interacting with the L1 and L2 providers.
#[derive(Clone, Debug)]
pub struct GraphClient {
    pub url: String,
    pub bearer_token: Option<String>,
    pub client: Client,
    pub provider: NormalProvider,
}

impl GraphClient {
    pub fn new(provider: NormalProvider, url: &str, bearer_token: Option<String>) -> Self {
        let client = Client::new();
        GraphClient {
            client,
            provider,
            url: url.to_string(),
            bearer_token,
        }
    }

    // get all deposited events by sender address
    pub async fn get_deposited_event_by_sender(
        &self,
        deposit_address: Address,
    ) -> Result<Vec<Deposited>, GraphClientError> {
        let query = r#"
        query MyQuery($senderAddress: String!) {
        depositeds(where: { sender: $senderAddress }) {
            sender
            tokenIndex
            transactionHash
            depositId
            recipientSaltHash
            amount
        }
        }
        "#;
        let request = json!({
            "query": query,
            "variables": {
                "senderAddress": deposit_address,
            }
        });
        let response: GraphQLResponse<DepositedsData> = post_request_with_bearer_token(
            &self.url,
            "",
            self.bearer_token.clone(),
            Some(&request),
        )
        .await?;

        let tx_hashes = response
            .data
            .depositeds
            .iter()
            .map(|entry| entry.transaction_hash)
            .collect::<Vec<TxHash>>();
        let txs = get_batch_transaction(&self.provider, &tx_hashes).await?;

        let mut deposited = Vec::new();
        for (event, tx) in response.data.depositeds.iter().zip(txs.iter()) {
            deposited.push(Deposited {
                sender: event.sender,
                token_index: event.token_index,
                deposit_id: event.deposit_id,
                recipient_salt_hash: event.recipient_salt_hash,
                amount: event.amount,
                tx_nonce: tx.nonce(),
            });
        }

        Ok(deposited)
    }

    pub async fn get_last_processed_deposit_id(
        &self,
        _deposit_address: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo!
        Ok(None)
    }

    pub async fn get_deposit_leaf_inserted_event(
        &self,
        _from_block: u64,
    ) -> Result<Vec<DepositLeafInserted>, GraphClientError> {
        // todo! fetch from the graph
        Ok(vec![])
    }

    pub async fn get_latest_deposit_timestamp(
        &self,
        _sender: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo! fetch from the graph
        Ok(None)
    }

    pub async fn get_latest_withdrawal_timestamp(
        &self,
        _recipient: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo! fetch from the graph
        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLResponse<T> {
    pub data: T,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositedsData {
    pub depositeds: Vec<DepositedEntry>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositedEntry {
    #[serde_as(as = "DisplayFromStr")]
    pub deposit_id: u64,
    pub sender: Address,
    pub recipient_salt_hash: Bytes32,
    #[serde_as(as = "DisplayFromStr")]
    pub token_index: u32,
    pub amount: U256,
    pub transaction_hash: TxHash,
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::external_api::contracts::utils::get_provider;

    #[tokio::test]
    async fn test_graph_client() {
        let graph_url =
            env::var("GRAPH_URL").unwrap_or_else(|_| "http://example.com/graphql".to_string());
        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
        let provider = get_provider(&rpc_url).unwrap();

        let client = GraphClient::new(provider, &graph_url, None);

        let result = client
            .get_deposited_event_by_sender(
                "0x4c5187eea6df32a4a2eadb3459a395c83309f0be"
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&result);
    }
}
