use std::{env, sync::Arc};

use ethers::{
    core::k256::{ecdsa::SigningKey, SecretKey},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{Address, Block, H256, U256},
    utils::hex::ToHex,
};
use log::info;

use crate::utils::retry::with_retry;

use super::error::BlockchainError;

fn get_rpc_url() -> Result<String, BlockchainError> {
    let rpc_url = env::var("RPC_URL")
        .map_err(|_| BlockchainError::EnvError("RPC_URL is not set".to_string()))?;
    Ok(rpc_url)
}

async fn get_provider() -> Result<Provider<Http>, BlockchainError> {
    let rpc_url = get_rpc_url()?;
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|_| BlockchainError::EnvError("Failed to parse RPC_URL".to_string()))?;
    Ok(provider)
}

pub async fn get_client() -> Result<Arc<Provider<Http>>, BlockchainError> {
    let provider = get_provider().await?;
    let client = Arc::new(provider);
    Ok(client)
}

pub async fn get_client_with_rpc_url(
    rpc_url: &str,
) -> Result<Arc<Provider<Http>>, BlockchainError> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| BlockchainError::InternalError(e.to_string()))?;
    Ok(Arc::new(provider))
}

pub async fn get_client_with_signer(
    private_key: H256,
) -> Result<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>, BlockchainError> {
    let provider = get_provider().await?;
    let wallet = get_wallet(private_key).await?;
    let client = SignerMiddleware::new(provider, wallet);
    Ok(client)
}

pub async fn get_wallet(private_key: H256) -> Result<Wallet<SigningKey>, BlockchainError> {
    let settings = crate::utils::config::Settings::load().unwrap();
    let key = SecretKey::from_bytes(private_key.as_bytes().into()).unwrap();
    let wallet = Wallet::from(key).with_chain_id(settings.blockchain.chain_id);
    Ok(wallet)
}

pub fn get_address(private_key: H256) -> Address {
    let wallet = private_key
        .encode_hex::<String>()
        .parse::<LocalWallet>()
        .unwrap();
    wallet.address()
}

// on chain queries
pub async fn get_account_nonce(address: Address) -> Result<u64, BlockchainError> {
    info!("get_account_nonce");
    let client = get_client().await?;
    let nonce = with_retry(|| async { client.get_transaction_count(address, None).await })
        .await
        .map_err(|_| BlockchainError::NetworkError("failed to get nonce".to_string()))?;
    Ok(nonce.as_u64())
}

pub async fn get_balance(address: Address) -> Result<U256, BlockchainError> {
    info!("get_balance");
    let client = get_client().await?;
    let balance = with_retry(|| async { client.get_balance(address, None).await })
        .await
        .map_err(|_| BlockchainError::NetworkError("failed to get balance".to_string()))?;
    Ok(balance)
}

pub async fn get_gas_price() -> Result<U256, BlockchainError> {
    info!("get_gas_price");
    let client = get_client().await?;
    let gas_price = with_retry(|| async { client.get_gas_price().await })
        .await
        .map_err(|_| BlockchainError::NetworkError("failed to get gas price".to_string()))?;
    Ok(gas_price)
}

pub async fn get_tx_receipt(
    tx_hash: H256,
) -> Result<ethers::core::types::TransactionReceipt, BlockchainError> {
    info!("get_tx_receipt");
    let client = get_client().await?;
    let mut loop_count = 0;
    loop {
        if loop_count > 20 {
            return Err(BlockchainError::TxNotFound(tx_hash.to_string()));
        }
        let receipt = with_retry(|| async { client.get_transaction_receipt(tx_hash).await })
            .await
            .map_err(|_| {
                BlockchainError::NetworkError("faied to get transaction receipt".to_string())
            })?;
        if receipt.is_some() {
            return Ok(receipt.unwrap());
        }
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        loop_count += 1;
    }
}

pub async fn get_block(block_number: u64) -> Result<Option<Block<H256>>, BlockchainError> {
    info!("get_block");
    let client = get_client().await.unwrap();
    let block = with_retry(|| async { client.get_block(block_number).await })
        .await
        .map_err(|_| BlockchainError::NetworkError("failed to get block".to_string()))?;
    Ok(block)
}

pub fn u256_as_bytes_be(u256: ethers::types::U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use ethers::types::Address;

    use crate::{external_api::contracts::token::get_token_balance, utils::config::Settings};

    #[tokio::test]
    async fn test_get_minter_token_balance() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        let settings = Settings::load()?;
        let minter_address: Address = settings.blockchain.minter_address.parse()?;
        let balance = get_token_balance(minter_address).await?;
        println!("{}", balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_gas_price() -> anyhow::Result<()> {
        dotenv::dotenv().ok();
        let gas_price = super::get_gas_price().await?;
        println!("{}", gas_price);
        Ok(())
    }
}
