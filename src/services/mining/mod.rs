use alloy::primitives::U256;
use anyhow::Context;

use withdrawal::withdrawal_task;

use crate::{
    cli::console::print_warning,
    external_api::contracts::convert::convert_u256_to_alloy,
    state::{key::Key, state::State},
    utils::errors::CLIError,
};

use super::{assets_status::AssetsStatus, utils::await_until_low_gas_price};

pub mod withdrawal;

pub async fn mining_task(
    state: &mut State,
    key: &Key,
    assets_status: &AssetsStatus,
    _new_deposit: bool,
    cancel_pending_deposits: bool,
    _mining_unit: U256,
) -> anyhow::Result<()> {
    // cancel pending deposits
    if cancel_pending_deposits {
        for &index in assets_status.pending_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            await_until_low_gas_price(&state.provider).await?;
            state
                .int1
                .cancel_deposit(
                    key.deposit_private_key,
                    event.deposit_id,
                    event.recipient_salt_hash,
                    event.token_index,
                    convert_u256_to_alloy(event.amount),
                )
                .await
                .context("Failed to cancel deposit")?;
        }
    }

    // cancel rejected deposits
    for &index in assets_status.rejected_indices.iter() {
        print_warning(format!(
            "Deposit address {:?} is rejected because of AML check. For more information, please refer to the documentation.",
            key.deposit_address
        ));
        let event = assets_status.senders_deposits[index].clone();
        await_until_low_gas_price(&state.provider).await?;
        state
            .int1
            .cancel_deposit(
                key.deposit_private_key,
                event.deposit_id,
                event.recipient_salt_hash,
                event.token_index,
                convert_u256_to_alloy(event.amount),
            )
            .await
            .context("Failed to cancel rejected deposit")?;
    }
    if !assets_status.rejected_indices.is_empty() {
        // Halt the CLI if a deposit is rejected to prevent further deposits
        return Err(CLIError::InternalError("Deposit is rejected".to_string()).into());
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
        return Ok(());
    }

    Ok(())
}
