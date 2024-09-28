use ethers::types::H256;

use crate::{
    external_api::contracts::utils::get_balance,
    services::utils::{is_address_used, pretty_format_u256},
    state::key::Key,
};

pub async fn export_deposit_accounts(withdrawal_private_key: H256) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            return Ok(());
        }
        let balance = get_balance(key.deposit_address).await?;
        println!();
        println!(
            "Deposit Address #{}: {:?} ({} ETH)",
            key_number,
            key.deposit_address,
            pretty_format_u256(balance),
        );
        println!("Private Key: {:?}", key.deposit_private_key);
        key_number += 1;
    }
}
