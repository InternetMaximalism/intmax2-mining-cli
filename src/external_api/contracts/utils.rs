use super::error::BlockchainError;
use alloy::{
    network::EthereumWallet,
    primitives::{Address, TxHash, B256},
    providers::{
        fillers::{
            ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, SimpleNonceManager,
            WalletFiller,
        },
        Identity, Provider, ProviderBuilder,
    },
    rpc::{client::RpcClient, types::Transaction},
    signers::local::PrivateKeySigner,
    transports::{
        http::Http,
        layers::{FallbackLayer, RetryBackoffLayer},
    },
};
use futures::{stream, StreamExt as _};
use reqwest::Url;
use std::{collections::HashMap, env};
use tower::ServiceBuilder;

// Use simple nonce manager for the nonce filler because it's easier to handle nonce errors.
pub type JoinedRecommendedFillersWithSimpleNonce = JoinFill<
    JoinFill<JoinFill<Identity, GasFiller>, NonceFiller<SimpleNonceManager>>,
    ChainIdFiller,
>;

pub type NormalProvider =
    FillProvider<JoinedRecommendedFillersWithSimpleNonce, alloy::providers::RootProvider>;

pub type ProviderWithSigner = FillProvider<
    JoinFill<JoinedRecommendedFillersWithSimpleNonce, WalletFiller<EthereumWallet>>,
    alloy::providers::RootProvider,
>;

// alloy does not support fallback transport in WASM, so we use a provider without fallback transport in WASM.
pub fn get_provider(rpc_urls: &str) -> Result<NormalProvider, BlockchainError> {
    let retry_layer = RetryBackoffLayer::new(5, 1000, 100);
    let url: Url = rpc_urls
        .parse()
        .map_err(|e| BlockchainError::ParseError(format!("Failed to parse URL {rpc_urls}: {e}")))?;
    let client = RpcClient::builder().layer(retry_layer).http(url);
    let provider = ProviderBuilder::default()
        .with_gas_estimation()
        .with_simple_nonce_management()
        .fetch_chain_id()
        .connect_client(client);
    Ok(provider)
}

pub fn get_provider_with_fallback(rpc_urls: &[String]) -> Result<NormalProvider, BlockchainError> {
    let retry_layer = RetryBackoffLayer::new(5, 1000, 100);
    let transports = rpc_urls
        .iter()
        .map(|url| {
            let url: Url = url.parse().map_err(|e| {
                BlockchainError::ParseError(format!("Failed to parse URL {url}: {e}"))
            })?;
            Ok(Http::new(url))
        })
        .collect::<Result<Vec<_>, BlockchainError>>()?;
    let fallback_layer =
        FallbackLayer::default().with_active_transport_count(transports.len().try_into().unwrap());
    let transport = ServiceBuilder::new()
        .layer(fallback_layer)
        .service(transports);
    let client = RpcClient::builder()
        .layer(retry_layer)
        .transport(transport, false);
    let provider = ProviderBuilder::default()
        .with_gas_estimation()
        .with_simple_nonce_management()
        .fetch_chain_id()
        .connect_client(client);
    Ok(provider)
}

pub fn get_provider_with_signer(
    provider: &NormalProvider,
    private_key: B256,
) -> ProviderWithSigner {
    let signer = PrivateKeySigner::from_bytes(&private_key).unwrap();
    let wallet = EthereumWallet::new(signer);
    let wallet_filler = WalletFiller::new(wallet);
    provider.clone().join_with(wallet_filler)
}

pub fn get_address_from_private_key(private_key: B256) -> Address {
    let signer = PrivateKeySigner::from_bytes(&private_key).unwrap();
    signer.address()
}

pub async fn get_batch_transaction(
    provider: &NormalProvider,
    tx_hashes: &[TxHash],
) -> Result<Vec<Transaction>, BlockchainError> {
    let mut target_tx_hashes = tx_hashes.to_vec();
    let mut fetched_txs = HashMap::new();
    let mut retry_count = 0;
    let max_tries = std::env::var("MAX_TRIES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    while !target_tx_hashes.is_empty() {
        let (partial_fetched_txs, failed_tx_hashes) =
            get_batch_transaction_inner(provider, &target_tx_hashes).await?;
        fetched_txs.extend(partial_fetched_txs);
        if failed_tx_hashes.is_empty() {
            break;
        }
        target_tx_hashes = failed_tx_hashes;
        retry_count += 1;
        if retry_count > max_tries {
            return Err(BlockchainError::TxNotFoundBatch);
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    let mut txs = Vec::new();
    for tx_hash in tx_hashes {
        txs.push(fetched_txs.get(tx_hash).unwrap().clone());
    }
    Ok(txs)
}

async fn get_batch_transaction_inner(
    provider: &NormalProvider,
    tx_hashes: &[TxHash],
) -> Result<(HashMap<TxHash, Transaction>, Vec<TxHash>), BlockchainError> {
    let max_parallel_requests = env::var("MAX_PARALLEL_REQUESTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let results = stream::iter(tx_hashes)
        .map(|&tx_hash| {
            let provider = provider.clone();
            async move {
                match provider.get_transaction_by_hash(tx_hash).await {
                    Ok(Some(tx)) => Ok((tx_hash, Ok(tx))),
                    Ok(None) => Ok((tx_hash, Err(BlockchainError::TxNotFound(tx_hash)))),
                    Err(e) => Err(e),
                }
            }
        })
        .buffer_unordered(max_parallel_requests)
        .collect::<Vec<_>>()
        .await;

    let mut fetched_txs = HashMap::new();
    let mut failed_tx_hashes = Vec::new();

    for result in results {
        match result {
            Ok((tx_hash, Ok(tx))) => {
                fetched_txs.insert(tx_hash, tx);
            }
            Ok((tx_hash, Err(BlockchainError::TxNotFound(_)))) => {
                failed_tx_hashes.push(tx_hash);
            }
            Ok((_, Err(e))) => return Err(e),
            Err(e) => return Err(BlockchainError::JoinError(e.to_string())),
        }
    }

    Ok((fetched_txs, failed_tx_hashes))
}
