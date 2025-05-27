use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, B256},
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};

use crate::utils::retry::with_retry;

use super::{
    error::BlockchainError,
    utils::{get_client, get_client_with_signer},
};

abigen!(MinterV1, "abi/MinterV1L.json",);

pub async fn get_minter_contract() -> Result<minter_v1::MinterV1<Provider<Http>>, BlockchainError> {
    let settings = crate::utils::config::Settings::load().unwrap();
    let client = get_client().await?;
    let minter_address: Address = settings.blockchain.minter_address.parse().unwrap();
    let contract = MinterV1::new(minter_address, client);
    Ok(contract)
}

pub async fn get_minter_contract_with_signer(
    private_key: B256,
) -> Result<
    minter_v1::MinterV1<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    BlockchainError,
> {
    let settings = crate::utils::config::Settings::load().unwrap();
    let client = get_client_with_signer(private_key).await?;
    let minter_address: Address = settings.blockchain.minter_address.parse().unwrap();
    let contract = MinterV1::new(minter_address, Arc::new(client));
    Ok(contract)
}

pub async fn get_short_term_eligible_root() -> Result<Bytes32, BlockchainError> {
    let minter = get_minter_contract().await?;
    let root = with_retry(|| async { minter.short_term_eligible_tree_root().call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call eligible_tree_root in minter".to_string())
        })?;
    Ok(Bytes32::from_bytes_be(&root))
}

pub async fn get_long_term_eligible_root() -> Result<Bytes32, BlockchainError> {
    let minter = get_minter_contract().await?;
    let root = with_retry(|| async { minter.long_term_eligible_tree_root().call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call eligible_tree_root in minter".to_string())
        })?;
    Ok(Bytes32::from_bytes_be(&root))
}

pub async fn get_short_term_claim_nullifier_exists(
    nullifier: Bytes32,
) -> Result<bool, BlockchainError> {
    let minter = get_minter_contract().await?;
    let nullifier: [u8; 32] = nullifier.to_bytes_be().try_into().unwrap();
    let exists = with_retry(|| async { minter.short_term_nullifiers(nullifier).call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call nullifiers in minter".to_string())
        })?;
    Ok(exists)
}

pub async fn get_long_term_claim_nullifier_exists(
    nullifier: Bytes32,
) -> Result<bool, BlockchainError> {
    let minter = get_minter_contract().await?;
    let nullifier: [u8; 32] = nullifier.to_bytes_be().try_into().unwrap();
    let exists = with_retry(|| async { minter.long_term_nullifiers(nullifier).call().await })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to call nullifiers in minter".to_string())
        })?;
    Ok(exists)
}
