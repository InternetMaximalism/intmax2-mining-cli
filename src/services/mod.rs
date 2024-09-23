use anyhow::Context;
use assets_status::fetch_assets_status;
use claim::claim_task;
use ethers::types::U256;
use mining::mining_task;

use crate::{
    cli::console::{print_assets_status, print_status},
    state::{
        keys::{ClaimKeys, MiningKeys},
        mode::RunMode,
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
    max_deposits: usize,
) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
        loop {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;

            if assets_status.senders_deposits.len() >= max_deposits {
                print_status("Max deposits reached. Exiting.");
                break;
            }

            let new_deposit = (assets_status.senders_deposits.len() < max_deposits) // deposit only if less than max deposits
            && (assets_status.pending_indices.is_empty()); // deposit only if no pending deposits
            mining_task(state, key, &assets_status, new_deposit, false, mining_uinit).await?;

            // print assets status
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;
            print_assets_status(&assets_status);

            main_loop_cooldown().await?;
        }
    }

    Ok(())
}

pub async fn exit_loop(state: &mut State, mining_keys: &MiningKeys) -> anyhow::Result<()> {
    for key in mining_keys.to_keys().iter() {
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
                print_status("All deposits are withdrawn. Exiting.");
                break;
            }

            mining_task(state, key, &assets_status, false, true, 0.into()).await?;

            // print assets status
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;
            print_assets_status(&assets_status);

            main_loop_cooldown().await?;
        }
    }

    Ok(())
}

pub async fn claim_loop(state: &mut State, claim_keys: ClaimKeys) -> anyhow::Result<()> {
    for key in claim_keys.to_keys().iter() {
        loop {
            state.sync_trees().await?;
            let assets_status =
                fetch_assets_status(&state, key.deposit_address, key.deposit_private_key)
                    .await
                    .context("Failed fetch assets status")?;

            if assets_status.not_claimed_indices.is_empty() {
                print_status("All eligible deposits are claimed. Claim process ended.");
                break;
            }

            claim_task(state, key, &assets_status).await?;

            // // print assets status
            // state.sync_trees().await?;
            // let assets_status = fetch_assets_status(
            //     &state.deposit_hash_tree,
            //     &state.eligible_tree,
            //     key.deposit_address,
            //     key.deposit_private_key,
            // )
            // .await
            // .context("Failed fetch assets status")?;
            // print_assets_status(&assets_status);

            // main_loop_cooldown().await?;
        }
    }

    Ok(())
}

/// Cooldown for main loop. `main_loop_cooldown_in_sec` seconds.
/// To avoid spamming RPC calls.
async fn main_loop_cooldown() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    tokio::time::sleep(std::time::Duration::from_secs(
        settings.service.main_loop_cooldown_in_sec,
    ))
    .await;
    Ok(())
}
