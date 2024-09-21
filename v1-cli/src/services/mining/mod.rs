use anyhow::Context;
use cancel::cancel_task;
use deposit::deposit_task;
use rand::Rng as _;
use withdrawal::withdrawal_task;

use crate::{cli::console::print_status, config::Settings, state::state::State};

use super::assets_status::AssetsStatus;

pub mod cancel;
pub mod deposit;
pub mod withdrawal;

pub async fn mining_task(
    state: &State,
    assets_status: &AssetsStatus,
    new_deposit: bool,
    cancel_pending_deposits: bool,
) -> anyhow::Result<()> {
    // cancel pending deposits
    if !assets_status.pending_indices.is_empty() && cancel_pending_deposits {
        for &index in assets_status.pending_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            cancel_task(state, event)
                .await
                .context("Failed to cancel pending deposits")?;
        }
    }

    // cancel rejected deposit_private_key
    if !assets_status.rejected_indices.is_empty() {
        for &index in assets_status.rejected_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            cancel_task(state, event)
                .await
                .context("Failed to cancel rejected deposit")?;
        }
    }

    // withdraw
    if !assets_status.not_withdrawn_indices.is_empty() {
        for &index in assets_status.not_withdrawn_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            withdrawal_task(state, event)
                .await
                .context("Failed withdrawal task")?;
        }
        mining_cooldown().await?;
        return Ok(());
    }

    // deposit
    if new_deposit {
        deposit_task(state).await.context("Failed deposit task")?;
        mining_cooldown().await?;
    }

    Ok(())
}

/// Cooldown for mining. Random time between 0 and `mining_max_cooldown_in_sec` to improve privacy.
async fn mining_cooldown() -> anyhow::Result<()> {
    print_status("Mining cooldown...");
    let settings = Settings::new()?;
    let cooldown = rand::thread_rng().gen_range(0..settings.service.mining_max_cooldown_in_sec);
    tokio::time::sleep(std::time::Duration::from_secs(cooldown)).await;
    Ok(())
}
