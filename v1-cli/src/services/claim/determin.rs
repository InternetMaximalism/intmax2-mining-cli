use intmax2_zkp::utils::leafable::Leafable as _;
use mining_circuit::claim::claim_inner_circuit::get_deposit_nullifier;

use crate::{
    external_api::contracts::{
        events::{get_deposited_event, DepositQuery, Deposited},
        minter::get_claim_nullifier_exists,
    },
    state::state::{RunMode, State},
    utils::salt::get_salt_from_private_key_nonce,
};

pub const MAX_CLAIMS: usize = 10;

#[derive(Debug, Clone)]
pub enum ClaimProcess {
    Claim(Vec<Deposited>),
    Wait,
    End,
}

pub async fn determin_next_claim_process(state: &State) -> anyhow::Result<ClaimProcess> {
    let deposit_address = state.private_data.to_addresses().await?.deposit_address;
    let all_senders_deposit_events =
        get_deposited_event(DepositQuery::BySender(deposit_address)).await?;

    // contained in the deposit tree
    let mut contained_deposit_events = Vec::new();
    for event in all_senders_deposit_events {
        if let Some(deposit_index) = state.deposit_hash_tree.get_index(event.deposit().hash()) {
            let mut event = event.clone();
            event.deposit_index = Some(deposit_index);
            contained_deposit_events.push(event);
        }
    }

    let mut contained_eligible_tree = Vec::new();
    for event in contained_deposit_events.iter() {
        if state
            .eligible_tree
            .get_leaf_index(event.deposit_index.unwrap())
            .is_some()
        {
            contained_eligible_tree.push(event);
        }
    }

    // check if there are any deposits that are not claimed.
    let deposit_key = state.private_data.deposit_key;
    let mut not_claimed_eligible_deposits = Vec::new();
    for event in contained_eligible_tree {
        let salt = get_salt_from_private_key_nonce(deposit_key, event.tx_nonce.unwrap());
        let nullifier = get_deposit_nullifier(&event.deposit(), salt);
        let is_exists = get_claim_nullifier_exists(nullifier).await?;
        if !is_exists {
            not_claimed_eligible_deposits.push(event.clone());
        }
    }

    if !not_claimed_eligible_deposits.is_empty() {
        // take at most MAX_CLAIMS events
        let not_claimed_eligible_deposits = not_claimed_eligible_deposits
            .into_iter()
            .take(MAX_CLAIMS)
            .collect::<Vec<_>>();
        return Ok(ClaimProcess::Claim(not_claimed_eligible_deposits));
    }

    match state.mode {
        RunMode::Normal => Ok(ClaimProcess::Wait),
        RunMode::Shutdown => Ok(ClaimProcess::End),
    }
}
