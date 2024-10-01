use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H256, U256},
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};

use crate::utils::retry::with_retry;

use super::{
    error::BlockchainError,
    utils::{get_client, get_client_with_signer},
};

abigen!(Int1, "abi/Int1.json",);

pub async fn get_int1_contract() -> Result<int_1::Int1<Provider<Http>>, BlockchainError> {
    let settings = crate::utils::config::Settings::load().unwrap();
    let int1_address: Address = settings.blockchain.int1_address.parse().unwrap();
    let client = get_client().await?;
    let contract = Int1::new(int1_address, client);
    Ok(contract)
}

pub async fn get_int1_contract_with_signer(
    private_key: H256,
) -> Result<int_1::Int1<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, BlockchainError> {
    let settings = crate::utils::config::Settings::load().unwrap();
    let client = get_client_with_signer(private_key).await?;
    let int1_address: Address = settings.blockchain.int1_address.parse().unwrap();
    let contract = Int1::new(int1_address, Arc::new(client));
    Ok(contract)
}

pub async fn get_deposit_root() -> Result<Bytes32, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let root = with_retry(|| async { int1.get_deposit_root().call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call get_deposit_root in int1".to_string())
        })?;
    Ok(Bytes32::from_bytes_be(&root))
}

pub async fn get_deposit_root_exits(root: Bytes32) -> Result<bool, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let root: [u8; 32] = root.to_bytes_be().try_into().unwrap();
    let block_number: U256 = with_retry(|| async { int1.deposit_roots(root).call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call deposit_roots in int1".to_string())
        })?;
    Ok(block_number != 0.into())
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DepositData {
    pub deposit_hash: Bytes32,
    pub sender: Address,
    pub is_rejected: bool,
}

pub async fn get_deposit_data(deposit_id: u64) -> Result<DepositData, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let data = with_retry(|| async {
        int1.get_deposit_data(ethers::types::U256::from(deposit_id))
            .call()
            .await
    })
    .await
    .map_err(|_| {
        BlockchainError::NetworkError("failed to call get_deposit_data in int1".to_string())
    })?;
    let data = DepositData {
        deposit_hash: Bytes32::from_bytes_be(&data.deposit_hash),
        sender: data.sender,
        is_rejected: data.is_rejected,
    };
    Ok(data)
}

pub async fn get_withdrawal_nullifier_exists(nullifier: Bytes32) -> Result<bool, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let nullifier: [u8; 32] = nullifier.to_bytes_be().try_into().unwrap();
    let block_number = with_retry(|| async { int1.nullifiers(nullifier).call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call nullifiers in int1".to_string())
        })?;
    let exists = block_number != 0.into();
    Ok(exists)
}

pub async fn get_last_processed_deposit_id() -> Result<u64, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let id = with_retry(|| async { int1.get_last_processed_deposit_id().call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError(
                "failed to call get_last_processed_deposit_id in int1".to_string(),
            )
        })?;
    Ok(id.as_u64())
}
