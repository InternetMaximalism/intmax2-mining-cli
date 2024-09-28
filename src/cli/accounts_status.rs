use ethers::types::H256;

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
};

/// Prints the status of the accounts
/// returns the key number where mining should start
pub async fn accounts_status(
    state: &mut State,
    mining_times: u64,
    withdrawal_private_key: H256,
) -> anyhow::Result<u64> {
    let withdrawal_address = get_address(withdrawal_private_key);
    let withdrawal_balance = get_balance(withdrawal_address).await?;
    let withdrawal_token_balance = get_token_balance(withdrawal_address).await?;
    println!(
        "Withdrawal address: {:?} {} ETH {} ITX",
        withdrawal_address,
        pretty_format_u256(withdrawal_balance),
        pretty_format_u256(withdrawal_token_balance),
    );

    let mut key_number = 0;
    let mut start_mining_key_number = 0;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            return Ok(start_mining_key_number);
        }
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        let is_qualified = !get_circulation(key.deposit_address).await?.is_excluded;
        let deposit_balance = get_balance(key.deposit_address).await?;
        println!(
            "Deposit address #{}: {:?} {} ETH. Qualified: {}. Deposits: {}/{}. Claimed: {}/{}",
            key_number,
            key.deposit_address,
            pretty_format_u256(deposit_balance),
            is_qualified,
            assets_status.senders_deposits.len(),
            mining_times,
            assets_status.claimed_indices.len(),
            assets_status.eligible_indices.len(),
        );
        if assets_status.senders_deposits.len() >= mining_times as usize {
            start_mining_key_number += 1;
        }
        key_number += 1;
    }
}
