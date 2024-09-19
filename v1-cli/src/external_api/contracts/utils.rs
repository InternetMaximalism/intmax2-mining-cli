use std::sync::Arc;

use ethers::{
    core::k256::{ecdsa::SigningKey, SecretKey},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
    types::{Address, H256, U256},
};

use crate::config::UserSettings;

pub async fn get_provider() -> anyhow::Result<Provider<Http>> {
    let user_settings = UserSettings::new()?;
    let provider = Provider::<Http>::try_from(user_settings.rpc_url)?;
    Ok(provider)
}

pub async fn get_client() -> anyhow::Result<Arc<Provider<Http>>> {
    let provider = get_provider().await?;
    let client = Arc::new(provider);
    Ok(client)
}

pub async fn get_client_with_signer(
    private_key: H256,
) -> anyhow::Result<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
    let provider = get_provider().await?;
    let wallet = get_wallet(private_key).await?;
    let client = SignerMiddleware::new(provider, wallet);
    Ok(client)
}

pub async fn get_wallet(private_key: H256) -> anyhow::Result<Wallet<SigningKey>> {
    let settings = crate::config::Settings::new()?;
    let key = SecretKey::from_be_bytes(private_key.as_bytes()).unwrap();
    let wallet = Wallet::from(key).with_chain_id(settings.blockchain.chain_id);
    Ok(wallet)
}

pub async fn get_account_nonce(address: Address) -> anyhow::Result<u64> {
    let client = get_client().await?;
    let nonce = client.get_transaction_count(address, None).await?;
    Ok(nonce.as_u64())
}

pub async fn get_balance(address: Address) -> anyhow::Result<U256> {
    let client = get_client().await?;
    let balance = client.get_balance(address, None).await?;
    Ok(balance)
}

pub async fn get_tx_receipt(
    tx_hash: H256,
) -> anyhow::Result<ethers::core::types::TransactionReceipt> {
    let client = get_client().await?;
    let receipt = client
        .get_transaction_receipt(tx_hash)
        .await?
        .ok_or(anyhow::anyhow!(
            "Transaction receipt not found for tx hash: {}",
            tx_hash
        ))?;
    Ok(receipt)
}

pub fn u256_as_bytes_be(u256: ethers::types::U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    bytes
}
