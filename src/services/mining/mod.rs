use alloy::{primitives::U256, providers::Provider};
use anyhow::Context;
use deterministic_sleep::{sleep_before_deposit, sleep_before_withdrawal};
use intmax2_zkp::common::deposit::get_pubkey_salt_hash;
use withdrawal::withdrawal_task;

use crate::{
    cli::console::print_warning,
    external_api::contracts::convert::convert_u256_to_alloy,
    state::{key::Key, state::State},
    utils::{
        derive_key::{derive_pubkey_from_private_key, derive_salt_from_private_key_nonce},
        errors::CLIError,
    },
};

use super::assets_status::AssetsStatus;

pub mod deterministic_sleep;
pub mod withdrawal;

pub async fn mining_task(
    state: &mut State,
    key: &Key,
    assets_status: &AssetsStatus,
    new_deposit: bool,
    cancel_pending_deposits: bool,
    mining_unit: U256,
) -> anyhow::Result<()> {
    // cancel pending deposits
    if cancel_pending_deposits {
        for &index in assets_status.pending_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
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
        // Halt the CLI if a deposit is rejected to prevent further deposits
        return Err(CLIError::InternalError("Deposit is rejected".to_string()).into());
    }

    // withdrawal
    if !assets_status.not_withdrawn_indices.is_empty() {
        sleep_before_withdrawal(&state.graph_client, key.deposit_address).await?;
        for &index in assets_status.not_withdrawn_indices.iter() {
            let event = assets_status.senders_deposits[index].clone();
            withdrawal_task(state, key, event)
                .await
                .map_err(|e| CLIError::InternalError(format!("Failed to withdrawal: {:#}", e)))?;
        }
        // return true to cooldown after withdrawal
        return Ok(());
    }

    // deposit
    if new_deposit {
        sleep_before_deposit(&state.graph_client, key.withdrawal_address).await?;
        let deposit_address = key.deposit_address;
        let account = state.provider.get_account(deposit_address).await?;
        let nonce = account.nonce;
        let salt = derive_salt_from_private_key_nonce(key.deposit_private_key, nonce);
        let pubkey = derive_pubkey_from_private_key(key.deposit_private_key);
        let pubkey_salt_hash = get_pubkey_salt_hash(pubkey, salt);
        // execute deposit task
        state
            .int1
            .deposit_native_token(key.deposit_private_key, pubkey_salt_hash, mining_unit)
            .await
            .context("Failed to deposit native token")?;
        return Ok(());
    }

    Ok(())
}
