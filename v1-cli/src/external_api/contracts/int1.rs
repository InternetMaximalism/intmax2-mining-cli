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

abigen!(Int1, "abi/Int1.json",);

pub async fn get_int1_contract() -> anyhow::Result<int_1::Int1<Provider<Http>>> {
    let settings = crate::config::Settings::new()?;
    let client = get_client().await?;
    let int1_address: Address = settings.blockchain.int1_address.parse()?;
    let contract = Int1::new(int1_address, client);
    Ok(contract)
}

pub async fn get_int1_contract_with_signer(
    private_key: H256,
) -> anyhow::Result<int_1::Int1<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>> {
    let settings = crate::config::Settings::new()?;
    let client = get_client_with_signer(private_key).await?;
    let int1_address: Address = settings.blockchain.int1_address.parse()?;
    let contract = Int1::new(int1_address, Arc::new(client));
    Ok(contract)
}

pub async fn get_deposit_root() -> anyhow::Result<Bytes32> {
    let int1 = get_int1_contract().await?;
    let root = int1.get_deposit_root().call().await?;
    Ok(Bytes32::from_bytes_be(&root))
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DepositData {
    pub deposit_hash: Bytes32,
    pub sender: Address,
    pub is_rejected: bool,
}

pub async fn get_deposit_data(deposit_id: u64) -> anyhow::Result<DepositData> {
    let int1 = get_int1_contract().await?;
    let data = int1
        .get_deposit_data(ethers::types::U256::from(deposit_id))
        .call()
        .await?;
    let data = DepositData {
        deposit_hash: Bytes32::from_bytes_be(&data.deposit_hash),
        sender: data.sender,
        is_rejected: data.is_rejected,
    };
    Ok(data)
}

pub async fn get_withdrawal_nullifier_exists(nullifier: Bytes32) -> anyhow::Result<bool> {
    let int1 = get_int1_contract().await?;
    let nullifier: [u8; 32] = nullifier.to_bytes_be().try_into().unwrap();
    let block_number = int1.nullifiers(nullifier).call().await?;
    let exists = block_number != 0.into();
    Ok(exists)
}
