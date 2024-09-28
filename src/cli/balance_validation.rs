use ethers::types::{Address, U256};

use crate::{
    external_api::contracts::utils::get_gas_price,
    services::{
        assets_status::AssetsStatus, claim::MAX_CLAIMS, utils::insuffient_balance_instruction,
    },
    utils::config::Settings,
};

pub async fn validate_deposit_address_balance(
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
    let gas_price = get_gas_price().await?;
    let single_deposit_gas: U256 = settings.blockchain.single_deposit_gas.into();
    let min_balance =
        (mining_unit + gas_price * single_deposit_gas) * U256::from(remaining_deposits);
    insuffient_balance_instruction(deposit_address, min_balance, "deposit").await?;
    Ok(())
}

pub async fn validate_withdrawal_address_balance(
    assets_status: &AssetsStatus,
    withdrawal_address: Address,
) -> anyhow::Result<()> {
    let remaining_claims = assets_status.not_claimed_indices.len();
    let num_claim_tx = (remaining_claims / MAX_CLAIMS) + 1;
    let settings = Settings::load()?;
    let gas_price = get_gas_price().await?;
    let single_claim_gas: U256 = settings.blockchain.single_claim_gas.into();
    let min_balance = single_claim_gas * gas_price * U256::from(num_claim_tx);
    insuffient_balance_instruction(withdrawal_address, min_balance, "withdrawal").await?;
    Ok(())
}
