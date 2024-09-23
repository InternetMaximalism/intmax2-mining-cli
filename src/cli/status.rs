use anyhow::Context as _;

use crate::{
    external_api::contracts::{token::get_token_balance, utils::get_balance},
    services::{assets_status::fetch_assets_status, contracts::pretty_format_u256},
    state::{private_data::PrivateData, state::State},
    utils::config::UserSettings,
};

pub async fn print_cli_status(state: &mut State, private_data: &PrivateData) -> anyhow::Result<()> {
    let deposit_balance = get_balance(private_data.deposit_address).await?;
    let claim_balance = get_balance(private_data.claim_address).await?;
    let claim_token_balance = get_token_balance(private_data.claim_address).await?;
    let withdrawal_balance = get_balance(private_data.withdrawal_address).await?;

    println!(
        "Deposit address: {:?} {} ETH\nClaim address: {:?} {} ETH {} ITX\nWithdrawal address: {:?} {} ETH",
        private_data.deposit_address,
        pretty_format_u256(deposit_balance),
        private_data.claim_address,
        pretty_format_u256(claim_balance),
        pretty_format_u256(claim_token_balance),
        private_data.withdrawal_address,
        pretty_format_u256(withdrawal_balance)
    );
    let max_deposits = UserSettings::new()?.max_deposits;
    state.sync_trees().await?;
    let assets_status = fetch_assets_status(
        &state.deposit_hash_tree,
        &state.eligible_tree,
        key.deposit_address,
        key.deposit_private_key,
    )
    .await
    .context("Failed fetch assets status")?;
    println!(
        "Current deposits / Max deposits: {} / {}",
        assets_status.senders_deposits.len(),
        max_deposits
    );
    println!();
    Ok(())
}
