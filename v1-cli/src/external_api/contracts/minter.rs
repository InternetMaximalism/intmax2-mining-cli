use std::sync::Arc;

use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, H256},
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};

use super::utils::{get_client, get_client_with_signer};

abigen!(MinterV1, "abi/MinterV1.json",);

pub async fn get_minter_contract() -> anyhow::Result<minter_v1::MinterV1<Provider<Http>>> {
    let settings = crate::config::Settings::new()?;
    let client = get_client().await?;
    let minter_address: Address = settings.blockchain.minter_address.parse()?;
    let contract = MinterV1::new(minter_address, client);
    Ok(contract)
}

pub async fn get_minter_contract_with_signer(
    private_key: H256,
) -> anyhow::Result<minter_v1::MinterV1<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let settings = crate::config::Settings::new()?;
    let client = get_client_with_signer(private_key).await?;
    let minter_address: Address = settings.blockchain.minter_address.parse()?;
    let contract = MinterV1::new(minter_address, Arc::new(client));
    Ok(contract)
}

pub async fn get_eligible_root() -> anyhow::Result<Bytes32> {
    let minter = get_minter_contract().await?;
    let root = minter.eligible_tree_root().call().await?;
    Ok(Bytes32::from_bytes_be(&root))
}

pub async fn get_claim_nullifier_exists(nullifier: Bytes32) -> anyhow::Result<bool> {
    let minter = get_minter_contract().await?;
    let nullifier: [u8; 32] = nullifier.to_bytes_be().try_into().unwrap();
    let exists = minter.nullifiers(nullifier).call().await?;
    Ok(exists)
}
