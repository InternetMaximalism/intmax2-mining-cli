use ethers::types::{H256, U256};

use crate::{
    external_api::{
        contracts::{
            token::get_token_balance,
            utils::{get_address, get_balance},
        },
        intmax::circulation::get_circulation,
    },
    services::utils::{is_address_used, pretty_format_u256},
    state::{key::Key, state::State},
    utils::network::get_network,
};

/// Prints the status of the accounts
/// returns the key number where mining should start
pub async fn accounts_status(
    state: &mut State,
    mining_times: u64,
    withdrawal_private_key: H256,
) -> anyhow::Result<()> {
    println!("Network: {}", get_network());
    let withdrawal_address = get_address(withdrawal_private_key);
    let withdrawal_balance = get_balance(withdrawal_address).await?;
    let withdrawal_token_balance = get_token_balance(withdrawal_address).await?;
    println!(
        "Withdrawal address(don’t deposit money to this): {} {} ETH {} ITX",
        withdrawal_address,
        pretty_format_u256(withdrawal_balance),
        pretty_format_u256(withdrawal_token_balance),
    );

    let mut key_number = 0;
    let mut total_claimable_amount = U256::zero();
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            println!(
                "Total claimable amount: {} ITX",
                pretty_format_u256(total_claimable_amount)
            );
            return Ok(());
        }
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        let is_qualified = !get_circulation(key.deposit_address).await?.is_excluded;
        let deposit_balance = get_balance(key.deposit_address).await?;
        println!(
            "Deposit address #{}: {:?} {} ETH. Qualified: {}. Deposits: {}/{}. Claimable: {} ITX",
            key_number,
            key.deposit_address,
            pretty_format_u256(deposit_balance),
            is_qualified,
            assets_status.senders_deposits.len(),
            mining_times,
            pretty_format_u256(assets_status.claimable_amount)
        );
        key_number += 1;
        total_claimable_amount += assets_status.claimable_amount;
    }
}
