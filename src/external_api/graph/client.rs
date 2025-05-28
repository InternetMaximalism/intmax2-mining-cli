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
        depositeds(
            where: { sender: $senderAddress },
            orderBy: blockTimestamp,
            orderDirection: asc
        ) {
            sender
            tokenIndex
            transactionHash
            blockTimestamp
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
                timestamp: event.block_timestamp,
            });
        }

        Ok(deposited)
    }

    pub async fn get_deposit_leaf_inserted_event(
        &self,
        next_deposit_index: u32,
    ) -> Result<Vec<DepositLeafInserted>, GraphClientError> {
        let limit = 1000; // Adjust the limit as needed
        let mut deposit_leaf_inserteds = Vec::new();
        let mut current_index = next_deposit_index;
        loop {
            let entries = self
                .get_deposit_leaf_inserted_event_inner(current_index, limit)
                .await?;
            if entries.is_empty() {
                break;
            }
            deposit_leaf_inserteds.extend(entries);
            current_index = deposit_leaf_inserteds.last().unwrap().deposit_index + 1;
        }
        Ok(deposit_leaf_inserteds)
    }

    async fn get_deposit_leaf_inserted_event_inner(
        &self,
        next_deposit_index: u32,
        limit: u64,
    ) -> Result<Vec<DepositLeafInserted>, GraphClientError> {
        let query = r#"
        query MyQuery($nextDepositIndex: BigInt!, $limit: Int!) {
        depositLeafInserteds(
            where: { depositIndex_gte: $nextDepositIndex },
            orderBy: depositIndex,
            orderDirection: asc,
            first: $limit
        ) {
            blockNumber
            depositIndex
            depositHash
        }
        }
        "#;
        let request = json!({
            "query": query,
            "variables": {
                "nextDepositIndex": next_deposit_index,
                "limit": limit,
            }
        });
        let response: GraphQLResponse<DepositLeafInsertedsData> = post_request_with_bearer_token(
            &self.url,
            "",
            self.bearer_token.clone(),
            Some(&request),
        )
        .await?;

        let deposit_leaf_inserteds = response
            .data
            .deposit_leaf_inserteds
            .into_iter()
            .map(|entry| DepositLeafInserted {
                deposit_index: entry.deposit_index,
                deposit_hash: entry.deposit_hash,
                block_number: entry.block_number,
            })
            .collect::<Vec<_>>();
        Ok(deposit_leaf_inserteds)
    }

    pub async fn get_latest_deposit_timestamp(
        &self,
        sender: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        let mut depositeds = self.get_deposited_event_by_sender(sender).await?;
        depositeds.sort_by_key(|d| d.timestamp);
        if let Some(latest_deposit) = depositeds.last() {
            Ok(Some(latest_deposit.timestamp))
        } else {
            Ok(None)
        }
    }

    pub async fn get_latest_withdrawal_timestamp(
        &self,
        recipient: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        let query = r#"
        query MyQuery($recipientAddress: String!) {
        withdrawns(
            where: { recipient: $recipientAddress },
            orderBy: blockTimestamp,
            orderDirection: desc,
            first: 1
        ) {
            blockTimestamp
        }
        }
        "#;
        let request = json!({
            "query": query,
            "variables": {
                "recipientAddress": recipient,
            }
        });
        let response: GraphQLResponse<WithdrawnData> = post_request_with_bearer_token(
            &self.url,
            "",
            self.bearer_token.clone(),
            Some(&request),
        )
        .await?;
        let timestamp = response
            .data
            .withdrawns
            .first()
            .map(|entry| entry.block_timestamp);
        Ok(timestamp)
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
    #[serde_as(as = "DisplayFromStr")]
    pub block_timestamp: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepositLeafInsertedsData {
    pub deposit_leaf_inserteds: Vec<DepositLeafInsertedEntry>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositLeafInsertedEntry {
    #[serde_as(as = "DisplayFromStr")]
    pub deposit_index: u32,
    pub deposit_hash: Bytes32,
    #[serde_as(as = "DisplayFromStr")]
    pub block_number: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawnData {
    pub withdrawns: Vec<WithdrawnEntry>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawnEntry {
    #[serde_as(as = "DisplayFromStr")]
    pub block_timestamp: u64,
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::external_api::contracts::utils::get_provider;

    fn get_client() -> Result<GraphClient, GraphClientError> {
        let graph_url =
            env::var("GRAPH_URL").unwrap_or_else(|_| "http://example.com/graphql".to_string());
        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
        let provider = get_provider(&rpc_url).unwrap();

        Ok(GraphClient::new(provider, &graph_url, None))
    }

    #[tokio::test]
    async fn test_graph_client_get_deposited_event_by_sender() {
        let client = get_client().unwrap();
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

    #[tokio::test]
    async fn test_graph_client_deposit_leaf_inserted_event() {
        let client = get_client().unwrap();
        let result = client.get_deposit_leaf_inserted_event(0).await.unwrap();
        dbg!(&result.len());
    }

    #[tokio::test]
    async fn test_graph_client_get_latest_deposit_timestamp() {
        let client = get_client().unwrap();
        let result = client
            .get_latest_deposit_timestamp(
                "0x4c5187eea6df32a4a2eadb3459a395c83309f0be"
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&result);
    }

    #[tokio::test]
    async fn test_graph_client_get_latest_withdrawal_timestamp() {
        let client = get_client().unwrap();
        let result = client
            .get_latest_withdrawal_timestamp(
                "0xC2233d8937d2581F374caB4C2E89257828bB1BF8"
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&result);
    }
}
