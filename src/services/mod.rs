use crate::{
    cli::{
        balance_validation::{
            validate_deposit_address_balance, validate_withdrawal_address_balance,
        },
        console::{print_assets_status, print_log, print_status},
    },
    state::{key::Key, state::State},
    utils::config::Settings,
};
use claim::claim_task;
use ethers::types::{H256, U256};
use mining::mining_task;
use rand::Rng as _;
use utils::is_address_used;

pub mod assets_status;
pub mod claim;
pub mod mining;
pub mod sync;
pub mod utils;

pub async fn mining_loop(
    state: &mut State,
    withdrawal_private_key: H256,
    start_key_number: u64,
    mining_unit: U256,
    mining_times: u64,
) -> anyhow::Result<()> {
    let mut key_number = start_key_number;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        print_log(format!(
            "Mining using deposit address #{} {:?}",
            key_number, key.deposit_address
        ));
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        validate_deposit_address_balance(
            &assets_status,
            key.deposit_address,
            mining_unit,
            mining_times,
        )
        .await?;
        loop {
            let assets_status = state.sync_and_fetch_assets(&key).await?;
            if assets_status.senders_deposits.len() >= mining_times as usize
                && assets_status.pending_indices.is_empty()
                && assets_status.rejected_indices.is_empty()
                && assets_status.not_withdrawn_indices.is_empty()
            {
                print_log(format!(
                    "Max deposits {} reached for {:?}.",
                    mining_times, key.deposit_address
                ));
                key_number += 1;
                break;
            }
            let new_deposit = (assets_status.senders_deposits.len() < mining_times as usize) // deposit only if less than max deposits
            && (assets_status.pending_indices.is_empty()); // deposit only if no pending deposits
            let cooldown =
                mining_task(state, &key, &assets_status, new_deposit, false, mining_unit).await?;

            // print assets status after mining
            let assets_status = state.sync_and_fetch_assets(&key).await?;
            print_assets_status(&assets_status);
            if cooldown {
                mining_cooldown().await?;
            }
            common_loop_cool_down().await;
        }
    }
}

pub async fn exit_loop(state: &mut State, withdrawal_private_key: H256) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            return Ok(());
        }
        print_status(format!(
            "Exit for deposit address #{} {:?}",
            key_number, key.deposit_address
        ));
        loop {
            let assets_status = state.sync_and_fetch_assets(&key).await?;
            if assets_status.pending_indices.is_empty()
                && assets_status.rejected_indices.is_empty()
                && assets_status.not_withdrawn_indices.is_empty()
            {
                print_status(format!(
                    "All deposits are withdrawn for {:?}.",
                    key.deposit_address,
                ));
                key_number += 1;
                break;
            }
            mining_task(state, &key, &assets_status, false, true, 0.into()).await?;

            common_loop_cool_down().await;
        }
    }
}

pub async fn claim_loop(state: &mut State, withdrawal_private_key: H256) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            return Ok(());
        }
        print_status(format!(
            "Claim for deposit address #{} {:?}",
            key_number, key.deposit_address
        ));
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        validate_withdrawal_address_balance(&assets_status, key.withdrawal_address).await?;

        let assets_status = state.sync_and_fetch_assets(&key).await?;
        claim_task(state, &key, &assets_status).await?;
        common_loop_cool_down().await;
        key_number += 1;
    }
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
