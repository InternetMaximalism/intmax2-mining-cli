use anyhow::Context;
use cancel::cancel_task;
use deposit::deposit_task;
use ethers::types::U256;
use withdrawal::withdrawal_task;

use crate::state::{keys::Key, state::State};

use super::assets_status::AssetsStatus;

pub mod cancel;
pub mod deposit;
pub mod withdrawal;

pub async fn mining_task(
    state: &State,
    key: &Key,
    assets_status: &AssetsStatus,
    new_deposit: bool,
    cancel_pending_deposits: bool,
    mining_unit: U256,
) -> anyhow::Result<bool> {
    // cancel pending deposits
    if !assets_status.pending_indices.is_empty() && cancel_pending_deposits {
        for &index in assets_status.pending_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            cancel_task(state, key, event)
                .await
                .context("Failed to cancel pending deposits")?;
        }
    }

    // cancel rejected deposits
    if !assets_status.rejected_indices.is_empty() {
        for &index in assets_status.rejected_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            cancel_task(state, key, event)
                .await
                .context("Failed to cancel rejected deposit")?;
        }
    }

    // withdraw
    if !assets_status.not_withdrawn_indices.is_empty() {
        for &index in assets_status.not_withdrawn_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            withdrawal_task(state, key, event)
                .await
                .context("Failed withdrawal task")?;
        }
        return Ok(true);
    }

    // deposit
    if new_deposit {
        deposit_task(state, key, mining_unit)
            .await
            .context("Failed deposit task")?;
        return Ok(true);
    }

    Ok(false)
}
