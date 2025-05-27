use crate::{
    cli::{
        availability::check_availability,
        balance_validation::{
            validate_deposit_address_balance, validate_withdrawal_address_balance,
        },
        console::{print_assets_status, print_log, print_status, print_warning},
    },
    external_api::intmax::circulation::get_circulation,
    state::{key::Key, state::State},
    utils::{config::Settings, time::sleep_for},
};
use alloy::primitives::{B256, U256};
use claim::claim_task;
use mining::mining_task;
use utils::is_address_used;

pub mod assets_status;
pub mod balance_transfer;
pub mod claim;
pub mod mining;
pub mod sync;
pub mod utils;

// note: in legacy environment, there is no mining loop
pub async fn mining_loop(
    state: &mut State,
    withdrawal_private_key: B256,
    mining_unit: U256,
    mining_times: u64,
) -> anyhow::Result<()> {
    let key = Key::new(withdrawal_private_key, 0);
    print_log(format!(
        "Processing mining for deposit address {:?}",
        key.deposit_address
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
        check_availability().await?;
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        let is_qualified = !get_circulation(key.deposit_address).await?.is_excluded;
        let will_deposit = assets_status.effective_deposit_times() < mining_times as usize
            && assets_status.pending_indices.is_empty()
            && is_qualified;

        // skip deposit address if no remaining deposits, and will not deposit
        if assets_status.no_remaining() && !will_deposit {
            if !is_qualified {
                print_warning(format!(
                        "Deposit address {:?}: is not qualified for mining. For more information, please refer to the documentation.",
                       key.deposit_address,
                    ));
            }
            print_log(format!(
                "Deposit address {:?}: Qualified: {}. Deposits {}/{}. Cancelled {}.",
                key.deposit_address,
                is_qualified,
                assets_status.effective_deposit_times(),
                mining_times,
                assets_status.cancelled_indices.len(),
            ));
            break;
        }
        mining_task(
            state,
            &key,
            &assets_status,
            will_deposit,
            false,
            mining_unit,
        )
        .await?;
        // print assets status after mining
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        print_assets_status(&assets_status);
        common_loop_cool_down();
    }
    print_log(format!(
        "Mining loop for deposit address {:?} ended",
        key.deposit_address
    ));
    Ok(())
}

pub async fn exit_loop(state: &mut State, withdrawal_private_key: B256) -> anyhow::Result<()> {
    let key = Key::new(withdrawal_private_key, 0);
    print_log(format!("Exit for deposit address{:?}", key.deposit_address));
    loop {
        check_availability().await?;
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        if assets_status.pending_indices.is_empty()
            && assets_status.rejected_indices.is_empty()
            && assets_status.not_withdrawn_indices.is_empty()
        {
            break;
        }
        mining_task(state, &key, &assets_status, false, true, 0.into()).await?;
        common_loop_cool_down();
    }
    print_status(format!(
        "Exit ended for deposit address {:?}",
        key.deposit_address,
    ));
    Ok(())
}

pub async fn legacy_exit_loop(
    state: &mut State,
    withdrawal_private_key: B256,
) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        check_availability().await?;
        let key = Key::new(withdrawal_private_key, key_number);
        if !is_address_used(key.deposit_address).await {
            print_status("exit loop finished".to_string());
            return Ok(());
        }
        print_log(format!(
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
                    "All deposits are withdrawn for #{}. {:?}",
                    key_number, key.deposit_address,
                ));
                key_number += 1;
                break;
            }
            mining_task(state, &key, &assets_status, false, true, 0.into()).await?;

            common_loop_cool_down();
        }
    }
}

pub async fn claim_loop(state: &mut State, withdrawal_private_key: B256) -> anyhow::Result<()> {
    let key = Key::new(withdrawal_private_key, 0);
    for is_short_term in [true, false] {
        check_availability().await?;
        if !is_address_used(key.deposit_address).await {
            print_status("claim loop finished".to_string());
            return Ok(());
        }
        print_log(format!(
            "Processing claim for term: {}",
            if is_short_term { "short" } else { "long" }
        ));
        let assets_status = state.sync_and_fetch_assets(&key).await?;
        validate_withdrawal_address_balance(&assets_status, key.withdrawal_address).await?;
        claim_task(state, &key, is_short_term, &assets_status).await?;
        common_loop_cool_down();
    }
    print_status(format!(
        "Claim ended for deposit address {:?}",
        key.deposit_address,
    ));
    Ok(())
}

pub async fn legacy_claim_loop(
    state: &mut State,
    withdrawal_private_key: B256,
) -> anyhow::Result<()> {
    let mut key_number = 0;
    loop {
        for is_short_term in [true, false] {
            check_availability().await?;
            let key = Key::new(withdrawal_private_key, key_number);
            if !is_address_used(key.deposit_address).await {
                print_status("claim loop finished".to_string());
                return Ok(());
            }
            print_log(format!(
                "Claim for deposit address #{} {:?}. Term: {}",
                key_number,
                key.deposit_address,
                if is_short_term { "short" } else { "long" }
            ));
            let assets_status = state.sync_and_fetch_assets(&key).await?;
            validate_withdrawal_address_balance(&assets_status, key.withdrawal_address).await?;
            let assets_status = state.sync_and_fetch_assets(&key).await?;
            claim_task(state, &key, is_short_term, &assets_status).await?;
            common_loop_cool_down();
        }
        key_number += 1;
    }
}

fn common_loop_cool_down() {
    let settings = Settings::load().expect("Failed to load settings");
    sleep_for(settings.service.loop_cooldown_in_sec);
}
