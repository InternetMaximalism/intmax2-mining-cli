use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u256::U256,
    utils::leafable::Leafable as _,
};

use crate::{
    config::UserSettings,
    external_api::contracts::{
        events::{get_deposited_event, DepositQuery, Deposited},
        int1::{get_deposit_data, get_withdrawal_nullifier_exists, DepositData},
    },
    state::state::{RunMode, State},
    utils::salt::get_salt_from_private_key_nonce,
};

#[derive(Debug, Clone)]
pub enum MiningProcess {
    Cancel(Deposited),
    Deposit,
    Withdrawal(Deposited),
    WaitingForAnalyze,
    EndBecauseOfNoRemainingDeposit,
    EndBecauseOfShutdown,
}

// determin which mining process should run next.
// 1. check if there are any deposits that are rejected. If so, cancel the deposit.
// 2. check if there are any deposits that are not withdrawn. If so, withdraw the deposit.
// 3. deposit
pub async fn determin_next_mining_process(state: &State) -> anyhow::Result<MiningProcess> {
    let deposit_address = state.private_data.to_addresses().await?.deposit_address;
    let all_senders_deposit_events =
        get_deposited_event(DepositQuery::BySender(deposit_address)).await?;

    // contained in the deposit tree
    let mut contained_deposit_events = Vec::new();

    // not contained in the deposit tree
    let mut not_contained_deposit_events = Vec::new();

    for event in all_senders_deposit_events {
        if let Some(deposit_index) = state.deposit_hash_tree.get_index(event.deposit().hash()) {
            let mut event = event.clone();
            event.deposit_index = Some(deposit_index);
            contained_deposit_events.push(event);
        } else {
            not_contained_deposit_events.push(event);
        }
    }

    let mut rejected_deposit_events = Vec::new();
    let mut cancelled_deposit_events = Vec::new();
    let mut pending_deposit_events = Vec::new();
    for event in &not_contained_deposit_events {
        let deposit_data = get_deposit_data(event.deposit_id).await?;
        if deposit_data.is_rejected {
            rejected_deposit_events.push(event.clone());
        } else if deposit_data == DepositData::default() {
            cancelled_deposit_events.push(event);
        } else {
            pending_deposit_events.push(event);
        }
    }

    if !pending_deposit_events.is_empty() {
        match state.mode {
            RunMode::Normal => {
                return Ok(MiningProcess::WaitingForAnalyze);
            }
            RunMode::Shutdown => {
                // cancel the deposit anyway
                return Ok(MiningProcess::Cancel(pending_deposit_events[0].clone()));
            }
        }
    }

    if !rejected_deposit_events.is_empty() {
        return Ok(MiningProcess::Cancel(rejected_deposit_events[0].clone()));
    }

    // check if there are any deposits that are not withdrawn.
    let deposit_key = state.private_data.deposit_key;
    let mut not_withdrawn_deposit_events = Vec::new();
    for event in contained_deposit_events {
        let salt = get_salt_from_private_key_nonce(deposit_key, event.tx_nonce.unwrap());
        let nullifier = get_pubkey_salt_hash(U256::default(), salt);
        let is_exists = get_withdrawal_nullifier_exists(nullifier).await?;
        if !is_exists {
            not_withdrawn_deposit_events.push(event);
        }
    }

    if !not_withdrawn_deposit_events.is_empty() {
        return Ok(MiningProcess::Withdrawal(
            not_withdrawn_deposit_events[0].clone(),
        ));
    }

    // if there is nothing to be cancelled or withdrawan, deposit if the mode is normal,
    // otherwise end the mining process.
    match state.mode {
        RunMode::Normal => {
            let user_settings = UserSettings::new()?;
            if user_settings.remaining_deposits == 0 {
                return Ok(MiningProcess::EndBecauseOfNoRemainingDeposit);
            } else {
                return Ok(MiningProcess::Deposit);
            }
        }
        RunMode::Shutdown => Ok(MiningProcess::EndBecauseOfShutdown),
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_determin_next_mining_process() {
        let mut state = crate::test::get_dummy_state();
        state.sync_trees().await.unwrap();

        let result = super::determin_next_mining_process(&state).await.unwrap();
        dbg!(result);
    }
}
