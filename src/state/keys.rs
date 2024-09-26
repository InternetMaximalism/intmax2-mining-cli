use ethers::types::{Address, H256};
use serde::{Deserialize, Serialize};

use crate::external_api::contracts::utils::get_address;

#[derive(Debug, Clone)]
pub struct Key {
    pub deposit_private_key: H256,
    pub deposit_address: Address,
    pub withdrawal_private_key: H256,
    pub withdrawal_address: Address,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keys {
    pub deposit_private_keys: Vec<H256>,
    pub deposit_addresses: Vec<Address>,
    pub withdrawal_private_key: H256,
    pub withdrawal_address: Address,
}

impl Keys {
    pub fn new(deposit_private_keys: Vec<H256>, withdrawal_private_key: H256) -> Self {
        let mut deposit_addresses = Vec::new();
        for key in deposit_private_keys.iter() {
            let address = get_address(*key);
            deposit_addresses.push(address);
        }
        let withdrawal_address = get_address(withdrawal_private_key);
        Self {
            deposit_private_keys,
            deposit_addresses,
            withdrawal_address,
            withdrawal_private_key,
        }
    }

    pub fn to_keys(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        for (deposit_private_key, deposit_address) in self
            .deposit_private_keys
            .iter()
            .zip(self.deposit_addresses.iter())
        {
            keys.push(Key {
                deposit_private_key: *deposit_private_key,
                deposit_address: *deposit_address,
                withdrawal_address: self.withdrawal_address,
                withdrawal_private_key: self.withdrawal_private_key,
            });
        }
        keys
    }
}
