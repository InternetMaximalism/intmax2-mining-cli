use ethers::types::{Address, U256};

use crate::{
    external_api::contracts::{
        token::get_token_balance,
        utils::{get_balance, get_gas_price},
    },
    services::{
        assets_status::{fetch_assets_status, AssetsStatus},
        claim::MAX_CLAIMS,
        contracts::pretty_format_u256,
    },
    state::state::State,
    utils::{config::Settings, errors::CLIError},
};

use super::load_env::Config;

pub async fn balance_validation(state: &mut State, config: Config) -> anyhow::Result<()> {
    let deposit_private_keys = match &config {
        Config::Mining(mining_config) => &mining_config.keys.deposit_private_keys,
        Config::Claim(claim_config) => &claim_config.keys.deposit_private_keys,
        Config::Exit(exit_config) => &exit_config.keys.deposit_private_keys,
    };
    let deposit_addresses: Vec<Address> = match &config {
        Config::Mining(mining_config) => mining_config.keys.deposit_addresses.clone(),
        Config::Claim(claim_config) => claim_config.keys.deposit_addresses.clone(),
        Config::Exit(exit_config) => exit_config.keys.deposit_addresses.clone(),
    };

    if let Config::Mining(mining_config) = config.clone() {
        for (&deposit_private_key, &deposit_address) in
            deposit_private_keys.iter().zip(deposit_addresses.iter())
        {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(state, deposit_address, deposit_private_key).await?;
            validate_deposit_address_balance(
                &assets_status,
                deposit_address,
                mining_config.mining_unit,
                mining_config.mining_times,
            )
            .await?;
        }
    } else if let Config::Claim(claim_config) = config.clone() {
        for (&deposit_private_key, &deposit_address) in
            deposit_private_keys.iter().zip(deposit_addresses.iter())
        {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(state, deposit_address, deposit_private_key).await?;
            validate_claim_address_balance(&assets_status, claim_config.keys.claim_address).await?;
        }
    }
    Ok(())
}

pub async fn validate_deposit_address_balance(
    assets_status: &AssetsStatus,
    deposit_address: Address,
    mining_unit: U256,
    mining_times: usize,
) -> anyhow::Result<()> {
    let num_deposits = assets_status.senders_deposits.len();
    let remaining_deposits = if mining_times > num_deposits {
        mining_times - num_deposits
    } else {
        0
    };

    let settings = Settings::new()?;
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
    println!(
        "Deposit address: {:?} Deposits: {}/{} Balance {} ETH",
        deposit_address,
        num_deposits,
        mining_times,
        pretty_format_u256(balance)
    );
    Ok(())
}

pub async fn validate_claim_address_balance(
    assets_status: &AssetsStatus,
    claim_address: Address,
) -> anyhow::Result<()> {
    let remaining_claims = assets_status.not_claimed_indices.len();
    let num_claim_tx = (remaining_claims / MAX_CLAIMS) + 1;
    let settings = Settings::new()?;
    let gas_price = get_gas_price().await?;
    let single_claim_gas: U256 = settings.blockchain.single_claim_gas.into();
    let min_balance = single_claim_gas * gas_price * U256::from(num_claim_tx);
    let balance = get_balance(claim_address).await?;
    if balance < min_balance {
        return Err(CLIError::BalanceError(format!(
            "Insufficient balance for claim address {:?}: current {}ETH < required {} ETH",
            claim_address,
            pretty_format_u256(balance),
            pretty_format_u256(min_balance)
        ))
        .into());
    }

    let balance = get_balance(claim_address).await?;
    let token_balance = get_token_balance(claim_address).await?;
    println!(
        "Claim address: {:?} Unclaimed: {} Balance: {} ETH {} ITX",
        claim_address,
        assets_status.not_claimed_indices.len(),
        pretty_format_u256(balance),
        pretty_format_u256(token_balance)
    );
    Ok(())
}
