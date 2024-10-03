use cancel::cancel_task;
use deposit::deposit_task;
use ethers::types::U256;
use withdrawal::withdrawal_task;

use crate::{
    cli::console::print_warning,
    state::{key::Key, state::State},
    utils::errors::CLIError,
};

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
    if cancel_pending_deposits {
        for &index in assets_status.pending_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            cancel_task(state, key, event).await.map_err(|e| {
                CLIError::InternalError(format!("Failed to cancel a pending deposit: {:#}", e))
            })?;
        }
    }

    // cancel rejected deposits
    for &index in assets_status.rejected_indices.iter() {
        print_warning(format!(
            "Deposit address {:?} is rejected because of AML check. For more information, please refer to the documentation.",
         key.deposit_address
        ));
        let event = assets_status.senders_deposits[index].clone();
        cancel_task(state, key, event).await.map_err(|e| {
            CLIError::InternalError(format!("Failed to cancel a rejected deposit: {:#}", e))
        })?;
    }

    // withdrawal
    if !assets_status.not_withdrawn_indices.is_empty() {
        for &index in assets_status.not_withdrawn_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            withdrawal_task(state, key, event)
                .await
                .map_err(|e| CLIError::InternalError(format!("Failed to withdrawal: {:#}", e)))?;
        }
        // return true to cooldown after withdrawal
        return Ok(true);
    }

    // deposit
    if new_deposit {
        deposit_task(state, key, mining_unit)
            .await
            .map_err(|e| CLIError::InternalError(format!("Failed to deposit: {:#}", e)))?;
        return Ok(true);
    }

    Ok(false)
}
