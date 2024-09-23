use anyhow::Context;
use assets_status::fetch_assets_status;
use claim::claim_task;
use ethers::types::U256;
use mining::mining_task;
use rand::Rng as _;

use crate::{
    cli::console::{print_assets_status, print_status},
    state::{
        keys::{ClaimKeys, MiningKeys},
        state::State,
    },
    utils::config::Settings,
};

pub mod assets_status;
pub mod claim;
pub mod contracts;
pub mod mining;
pub mod sync;

pub async fn mining_loop(
    state: &mut State,
    mining_keys: &MiningKeys,
    mining_uinit: U256,
    mining_times: usize,
) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
        print_status(format!("Mining loop for {:?}", key.deposit_address));
        loop {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;

            if assets_status.senders_deposits.len() >= mining_times {
                print_status(format!(
                    "Max deposits reached for {:?}. Exiting.",
                    key.deposit_address
                ));
                break;
            }

            let new_deposit = (assets_status.senders_deposits.len() < mining_times) // deposit only if less than max deposits
            && (assets_status.pending_indices.is_empty()); // deposit only if no pending deposits
            let cooldown =
                mining_task(state, key, &assets_status, new_deposit, false, mining_uinit).await?;

            // print assets status
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;
            print_assets_status(&assets_status);

            if cooldown {
                mining_cooldown().await?;
            }
        }
    }

    Ok(())
}

pub async fn exit_loop(state: &mut State, mining_keys: &MiningKeys) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
        print_status(format!("Exit loop for {:?}", key.deposit_address));
        loop {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;

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
        }
    }

    Ok(())
}

pub async fn claim_loop(state: &mut State, claim_keys: ClaimKeys) -> anyhow::Result<()> {
    for key in claim_keys.to_keys().iter() {
        print_status(format!("Claim loop for {:?}", key.deposit_address));
        loop {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;

            if assets_status.not_claimed_indices.is_empty() {
                print_status(format!(
                    "All eligible deposits are claimed for {:?}. Claim process ended.",
                    key.deposit_address
                ));
                break;
            }

            claim_task(state, key, &assets_status).await?;
        }
    }

    Ok(())
}

/// Cooldown for mining. Random time between 0 and `mining_max_cooldown_in_sec` to improve privacy.
async fn mining_cooldown() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    let cooldown = rand::thread_rng().gen_range(0..settings.service.mining_max_cooldown_in_sec);
    tokio::time::sleep(std::time::Duration::from_secs(cooldown)).await;
    Ok(())
}
