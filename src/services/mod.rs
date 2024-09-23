use anyhow::Context;
use assets_status::fetch_assets_status;
use claim::claim_task;
use mining::mining_task;

use crate::{
    cli::console::{print_assets_status, print_status},
    state::{mode::RunMode, state::State},
    utils::config::{Settings, UserSettings},
};

pub mod assets_status;
pub mod claim;
pub mod contracts;
pub mod mining;
pub mod sync;

pub async fn main_loop(state: &mut State) -> anyhow::Result<()> {
    let max_deposits = UserSettings::new()?.max_deposits;

    loop {
        state.sync_trees().await?;
        let assets_status = fetch_assets_status(
            &state.deposit_hash_tree,
            &state.eligible_tree,
            key.deposit_address,
            key.deposit_private_key,
        )
        .await
        .context("Failed fetch assets status")?;

        if state.mode == RunMode::Exit
            && assets_status.pending_indices.is_empty()
            && assets_status.rejected_indices.is_empty()
            && assets_status.not_withdrawn_indices.is_empty()
        {
            print_status("All deposits are withdrawn. Exiting.");
            break;
        }

        if state.mode == RunMode::Mining && assets_status.senders_deposits.len() >= max_deposits {
            print_status("Max deposits reached. Mining and Claim process ended.");
            break;
        }

        if state.mode == RunMode::Claim && assets_status.not_claimed_indices.is_empty() {
            print_status("All eligible deposits are claimed. Claim process ended.");
            break;
        }

        if state.mode == RunMode::Mining || state.mode == RunMode::Exit {
            let new_deposit = (assets_status.senders_deposits.len() < max_deposits) // deposit only if less than max deposits
            && (assets_status.pending_indices.is_empty()) // deposit only if no pending deposits
            && (state.mode != RunMode::Exit); // do not deposit in shutdown mode
            let canncel_pending_deposits = state.mode == RunMode::Exit;
            mining_task(state, &assets_status, new_deposit, canncel_pending_deposits).await?;
        }

        if state.mode == RunMode::Claim {
            claim_task(state, &assets_status).await?;
        }

        // print assets status
        state.sync_trees().await?;
        let assets_status = fetch_assets_status(
            &state.deposit_hash_tree,
            &state.eligible_tree,
            key.deposit_address,
            key.deposit_private_key,
        )
        .await
        .context("Failed fetch assets status")?;
        print_assets_status(&assets_status);

        main_loop_cooldown().await?;
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
