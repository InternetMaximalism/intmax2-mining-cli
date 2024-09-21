use std::sync::Arc;

use anyhow::ensure;
use ethers::{
    core::k256::{ecdsa::SigningKey, SecretKey},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
    types::{Address, H256, U256},
};

use crate::{
    cli::console::{insuffient_balance_instruction, print_status},
    config::UserSettings,
};

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

pub async fn get_client_with_rpc_url(rpc_url: &str) -> anyhow::Result<Arc<Provider<Http>>> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    Ok(Arc::new(provider))
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
    let mut loop_count = 0;
    loop {
        if loop_count > 10 {
            return Err(anyhow::anyhow!(
                "Transaction not mined for tx hash: {:?}",
                tx_hash
            ));
        }
        let receipt = client.get_transaction_receipt(tx_hash).await?;
        if receipt.is_some() {
            return Ok(receipt.unwrap());
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        loop_count += 1;
    }
}

pub fn u256_as_bytes_be(u256: ethers::types::U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    u256.to_big_endian(&mut bytes);
    bytes
}

pub async fn handle_contract_call<S: ToString>(
    tx: ethers::contract::builders::ContractCall<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
    nonce: Option<u64>,
    from_address: Address,
    from_name: S,
    tx_name: S,
) -> anyhow::Result<()> {
    loop {
        let mut tx_with_nonce = tx.clone();
        if let Some(nonce) = nonce {
            tx_with_nonce.tx.set_nonce(nonce);
        }
        let result = tx_with_nonce.send().await;
        match result {
            Ok(tx) => {
                let pending_tx = tx;
                print_status(format!(
                    "{} tx hash: {:?}",
                    tx_name.to_string(),
                    pending_tx.tx_hash()
                ));
                let tx_receipt = pending_tx.await?.unwrap();
                ensure!(
                    tx_receipt.status.unwrap() == 1.into(),
                    "{} tx failed",
                    from_name.to_string()
                );
                return Ok(());
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("-32000") {
                    insuffient_balance_instruction(from_address, &from_name.to_string()).await?;
                    print_status(format!("Retrying {} transaction...", tx_name.to_string()));
                } else {
                    return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
                }
            }
        }
    }
}
