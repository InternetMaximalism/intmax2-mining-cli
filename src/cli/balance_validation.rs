use ethers::types::{Address, U256};

use crate::{
    external_api::{
        contracts::{
            token::get_token_balance,
            utils::{get_balance, get_gas_price},
        },
        intmax::circulation::get_circulation,
    },
    services::{assets_status::AssetsStatus, claim::MAX_CLAIMS, contracts::pretty_format_u256},
    state::{key::Keys, mode::RunMode, state::State},
    utils::{config::Settings, env_config::EnvConfig, errors::CLIError},
};

pub async fn balance_validation(
    state: &mut State,
    mode: RunMode,
    config: &EnvConfig,
    keys: &Keys,
) -> anyhow::Result<()> {
    if mode == RunMode::Mining {
        for key in keys.to_keys().iter() {
            let assets_status = state.sync_and_fetch_assets(key).await?;
            validate_deposit_address_balance(
                &assets_status,
                key.deposit_address,
                config.mining_unit,
                config.mining_times,
            )
            .await?;
        }
    } else if mode == RunMode::Claim {
        for key in keys.to_keys().iter() {
            let assets_status = state.sync_and_fetch_assets(key).await?;
            validate_withdrawal_address_balance(&assets_status, keys.withdrawal_address).await?;
        }
    }
    Ok(())
}

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
    let balance = get_balance(deposit_address).await?;
    if balance < min_balance {
        return Err(CLIError::BalanceError(format!(
            "Insufficient balance for deposit address {:?}: current {}ETH < required {} ETH",
            deposit_address,
            pretty_format_u256(balance),
            pretty_format_u256(min_balance)
        ))
        .into());
    }
    let is_not_reward_target = get_circulation(deposit_address).await?.is_excluded;
    if is_not_reward_target {
        return Err(CLIError::CirculationError(format!(
            "Deposit address {:?} is excluded from rewards",
            deposit_address
        ))
        .into());
    }
    println!(
        "Deposit address: {:?} Deposits: {}/{} Balance {} ETH",
        deposit_address,
        num_deposits,
        mining_times,
        pretty_format_u256(balance)
    );
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
    let balance = get_balance(withdrawal_address).await?;
    if balance < min_balance {
        return Err(CLIError::BalanceError(format!(
            "Insufficient balance for withdrawal address {:?}: current {}ETH < required {} ETH",
            withdrawal_address,
            pretty_format_u256(balance),
            pretty_format_u256(min_balance)
        ))
        .into());
    }
    let balance = get_balance(withdrawal_address).await?;
    let token_balance = get_token_balance(withdrawal_address).await?;
    println!(
        "Withdawal address: {:?} Unclaimed: {} Balance: {} ETH {} ITX",
        withdrawal_address,
        remaining_claims,
        pretty_format_u256(balance),
        pretty_format_u256(token_balance)
    );
    Ok(())
}
