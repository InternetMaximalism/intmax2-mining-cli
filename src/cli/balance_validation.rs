use alloy::{
    primitives::{Address, U256},
    providers::Provider,
};

use crate::{
    external_api::contracts::utils::NormalProvider,
    services::{
        assets_status::AssetsStatus, claim::MAX_CLAIMS, utils::insufficient_balance_instruction,
    },
    utils::config::Settings,
};

pub async fn validate_deposit_address_balance(
    provider: &NormalProvider,
    assets_status: &AssetsStatus,
    deposit_address: Address,
    mining_unit: U256,
    mining_times: u64,
) -> anyhow::Result<()> {
    let num_deposits = assets_status.senders_deposits.len() as u64;
    let remaining_deposits = if mining_times > num_deposits {
        mining_times - num_deposits
    } else {
        0
    };
    let settings = Settings::load()?;
    let gas_price = U256::from(provider.get_gas_price().await?);
    let single_deposit_gas = U256::from(settings.blockchain.single_deposit_gas);
    let min_balance =
        (mining_unit + gas_price * single_deposit_gas) * U256::from(remaining_deposits);
    insufficient_balance_instruction(deposit_address, min_balance, "deposit").await?;
    Ok(())
}

pub async fn validate_withdrawal_address_balance(
    provider: &NormalProvider,
    assets_status: &AssetsStatus,
    withdrawal_address: Address,
) -> anyhow::Result<()> {
    let remaining_claims = assets_status.short_term_not_claimed_indices.len();
    let num_claim_tx = remaining_claims.div_ceil(MAX_CLAIMS);
    let settings = Settings::load()?;
    let gas_price = U256::from(provider.get_gas_price().await?);
    let single_claim_gas = U256::from(settings.blockchain.single_claim_gas);
    let min_balance = single_claim_gas * gas_price * U256::from(num_claim_tx);
    insufficient_balance_instruction(withdrawal_address, min_balance, "withdrawal").await?;
    Ok(())
}
