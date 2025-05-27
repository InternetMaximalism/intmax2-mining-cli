use ethers::types::{Address, B256};

use crate::{
    external_api::contracts::utils::get_address, utils::derive_key::derive_deposit_private_key,
};

#[derive(Debug, Clone)]
pub struct Key {
    pub deposit_private_key: B256,
    pub deposit_address: Address,
    pub withdrawal_private_key: B256,
    pub withdrawal_address: Address,
}

impl Key {
    pub fn new(withdrawal_private_key: B256, number: u64) -> Self {
        let withdrawal_address = get_address(withdrawal_private_key);
        let deposit_private_key = derive_deposit_private_key(withdrawal_private_key, number);
        let deposit_address = get_address(deposit_private_key);
        Self {
            deposit_private_key,
            deposit_address,
            withdrawal_private_key,
            withdrawal_address,
        }
    }
}
