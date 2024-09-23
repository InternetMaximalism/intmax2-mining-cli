use ethers::types::{Address, H256};

use crate::external_api::contracts::utils::get_address;

#[derive(Debug, Clone)]
pub struct Key {
    pub deposit_private_key: H256,
    pub deposit_address: Address,
    pub claim_private_key: Option<H256>,
    pub claim_address: Option<Address>,
    pub withdrawal_address: Option<Address>,
}

#[derive(Debug, Clone)]
pub struct MiningKeys {
    pub deposit_private_keys: Vec<H256>,
    pub deposit_addresses: Vec<Address>,
    pub withdrawal_address: Address,
}

impl MiningKeys {
    pub async fn new(deposit_private_keys: Vec<H256>, withdrawal_address: Address) -> Self {
        let mut deposit_addresses = Vec::new();
        for key in deposit_private_keys.iter() {
            let address = get_address(*key).await;
            deposit_addresses.push(address);
        }
        Self {
            deposit_private_keys,
            deposit_addresses,
            withdrawal_address,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClaimKeys {
    pub deposit_private_keys: Vec<H256>,
    pub deposit_addresses: Vec<Address>,
    pub claim_private_key: H256,
    pub claim_address: Address,
}

impl ClaimKeys {
    pub async fn new(deposit_private_keys: Vec<H256>, claim_private_key: H256) -> Self {
        let mut deposit_addresses = Vec::new();
        for key in deposit_private_keys.iter() {
            let address = get_address(*key).await;
            deposit_addresses.push(address);
        }
        let claim_address = get_address(claim_private_key).await;
        Self {
            deposit_private_keys,
            deposit_addresses,
            claim_private_key,
            claim_address,
        }
    }
}
