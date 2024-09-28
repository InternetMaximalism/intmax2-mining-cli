use claim::claim_task;
use ethers::types::U256;
use mining::mining_task;
use rand::Rng as _;

use crate::{
    cli::console::{print_assets_status, print_status},
    state::{key::Keys, state::State},
    utils::{config::Settings, errors::CLIError},
};

pub mod assets_status;
pub mod claim;
pub mod contracts;
pub mod gas_validation;
pub mod mining;
pub mod sync;

pub async fn mining_loop(
    state: &mut State,
    mining_keys: &Keys,
    mining_uinit: U256,
    mining_times: u64,
) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
        print_status(format!("Mining loop for {:?}", key.deposit_address));
        loop {
            let assets_status = state.sync_and_fetch_assets(key).await.map_err(|e| {
                CLIError::NetworkError(format!(
                    "Failed while fetching assets status for {:?}: {:?}",
                    key.deposit_address, e
                ))
            })?;

            if assets_status.senders_deposits.len() >= mining_times as usize
                && assets_status.pending_indices.is_empty()
                && assets_status.rejected_indices.is_empty()
                && assets_status.not_withdrawn_indices.is_empty()
            {
                print_status(format!(
                    "Max deposits {} reached for {:?}. Please use another deposit address.",
                    mining_times, key.deposit_address
                ));
                break;
            }

            let new_deposit = (assets_status.senders_deposits.len() < mining_times as usize) // deposit only if less than max deposits
            && (assets_status.pending_indices.is_empty()); // deposit only if no pending deposits
            let cooldown =
                mining_task(state, key, &assets_status, new_deposit, false, mining_uinit).await?;

            // print assets status after mining
            let assets_status = state.sync_and_fetch_assets(key).await.map_err(|e| {
                CLIError::NetworkError(format!(
                    "Failed while fetching assets status for {:?}: {:?}",
                    key.deposit_address, e
                ))
            })?;
            print_assets_status(&assets_status);

            if cooldown {
                mining_cooldown().await?;
            }

            common_loop_cool_down().await;
        }
    }

    Ok(())
}

pub async fn exit_loop(state: &mut State, mining_keys: &Keys) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
        print_status(format!("Exit loop for {:?}", key.deposit_address));
        loop {
            let assets_status = state.sync_and_fetch_assets(key).await.map_err(|e| {
                CLIError::NetworkError(format!(
                    "Failed while fetching assets status for {:?}: {:?}",
                    key.deposit_address, e
                ))
            })?;

            if assets_status.pending_indices.is_empty()
                && assets_status.rejected_indices.is_empty()
                && assets_status.not_withdrawn_indices.is_empty()
            {
                print_status(format!(
                    "All deposits are withdrawn for {:?}. Exiting.",
                    key.deposit_address,
                ));
                break;
            }

            mining_task(state, key, &assets_status, false, true, 0.into()).await?;

            common_loop_cool_down().await;
        }
    }

    Ok(())
}

pub async fn claim_loop(state: &mut State, keys: &Keys) -> anyhow::Result<()> {
    for key in keys.to_keys().iter() {
        print_status(format!("Claim loop for {:?}", key.deposit_address));
        loop {
            let assets_status = state.sync_and_fetch_assets(key).await.map_err(|e| {
                CLIError::NetworkError(format!(
                    "Failed while fetching assets status for {:?}: {:?}",
                    key.deposit_address, e
                ))
            })?;

            if assets_status.not_claimed_indices.is_empty() {
                print_status(format!(
                    "All eligible rewards are claimed for {:?}.",
                    key.deposit_address
                ));
                break;
            }

            claim_task(state, key, &assets_status).await?;

            common_loop_cool_down().await;
        }
    }

    Ok(())
}

async fn common_loop_cool_down() {
    let settings = Settings::load().expect("Failed to load settings");
    tokio::time::sleep(std::time::Duration::from_secs(
        settings.service.loop_cooldown_in_sec,
    ))
    .await;
}

/// Cooldown for mining. Random time between 0 and `mining_max_cooldown_in_sec` to improve privacy.
async fn mining_cooldown() -> anyhow::Result<()> {
    let settings = Settings::load()?;
    let cooldown = rand::thread_rng().gen_range(0..settings.service.mining_max_cooldown_in_sec);
    tokio::time::sleep(std::time::Duration::from_secs(cooldown)).await;
    Ok(())
}
