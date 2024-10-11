use ethers::types::{Address, H256, U256};
use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u32limb_trait::U32LimbTrait,
    utils::leafable::Leafable as _,
};
use log::warn;
use mining_circuit_v1::claim::claim_inner_circuit::get_deposit_nullifier;

use crate::{
    external_api::contracts::{
        events::{get_deposited_event_by_sender, Deposited},
        int1::{
            get_deposit_data, get_last_processed_deposit_id, get_withdrawal_nullifier_exists,
            DepositData,
        },
        minter::{get_long_term_claim_nullifier_exists, get_short_term_claim_nullifier_exists},
    },
    state::state::State,
    utils::derive_key::derive_salt_from_private_key_nonce,
};

#[derive(Debug, Clone)]
pub struct AssetsStatus {
    pub senders_deposits: Vec<Deposited>,
    pub contained_indices: Vec<usize>, // Positions in senders_deposits that are contained in the deposit tree
    pub rejected_indices: Vec<usize>,  // Positions in senders_deposits that are rejected
    pub cancelled_indices: Vec<usize>, // Positions in senders_deposits that are cancelled
    pub pending_indices: Vec<usize>,   // Positions in senders_deposits that are not analyzed yet
    pub withdrawn_indices: Vec<usize>, // Positions in senders_deposits that are withdrawn
    pub not_withdrawn_indices: Vec<usize>, // Positions in senders_deposits that are contained but not withdrawn
    pub short_term_eligible_indices: Vec<usize>, // Positions in senders_deposits that are eligible
    pub short_term_claimed_indices: Vec<usize>, // Positions in senders_deposits that are claimed
    pub short_term_not_claimed_indices: Vec<usize>, // Positions in senders_deposits that are eligible but not claimed
    pub short_term_claimable_amount: U256,          // Total amount of not claimed tokens
    pub long_term_eligible_indices: Vec<usize>, // Positions in senders_deposits that are eligible
    pub long_term_claimed_indices: Vec<usize>,  // Positions in senders_deposits that are claimed
    pub long_term_not_claimed_indices: Vec<usize>, // Positions in senders_deposits that are eligible but not claimed
    pub long_term_claimable_amount: U256,          // Total amount of not claimed tokens
}

pub async fn fetch_assets_status(
    state: &State,
    deposit_address: Address,
    deposit_private_key: H256,
) -> anyhow::Result<AssetsStatus> {
    let senders_deposits = get_deposited_event_by_sender(deposit_address).await?;

    let mut contained_indices = Vec::new();
    let mut not_contained_indices = Vec::new();
    for (index, event) in senders_deposits.iter().enumerate() {
        if state
            .deposit_hash_tree
            .get_index(event.deposit().hash())
            .is_some()
        {
            contained_indices.push(index);
        } else {
            not_contained_indices.push(index);
        }
    }

    let mut rejected_indices = Vec::new();
    let mut cancelled_indices = Vec::new();
    let mut pending_indices = Vec::new();
    for &index in &not_contained_indices {
        let event = &senders_deposits[index];
        let deposit_data = get_deposit_data(event.deposit_id).await?;
        if deposit_data.is_rejected {
            rejected_indices.push(index);
        } else if deposit_data == DepositData::default() {
            cancelled_indices.push(index);
        } else {
            let last_processed_deposit_id = get_last_processed_deposit_id().await?;
            // this may occur because of the delay of the event log
            if event.deposit_id < last_processed_deposit_id {
                warn!(
                    "Deposit should have been processed: last_processed_deposit_id={}, deposit_id={}",
                    last_processed_deposit_id, event.deposit_id
                );
            }
            pending_indices.push(index);
        }
    }

    let mut withdrawn_indices = Vec::new();
    let mut not_withdrawn_indices = Vec::new();
    for &index in contained_indices.iter() {
        let event = &senders_deposits[index];
        let salt = derive_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce);
        let nullifier =
            get_pubkey_salt_hash(intmax2_zkp::ethereum_types::u256::U256::default(), salt);
        let is_exists = get_withdrawal_nullifier_exists(nullifier).await?;
        if is_exists {
            withdrawn_indices.push(index);
        } else {
            not_withdrawn_indices.push(index);
        }
    }

    let mut short_term_eligible_indices = Vec::new();
    let mut long_term_eligible_indices = Vec::new();
    let mut short_term_eligible_amounts = Vec::new();
    let mut long_term_eligible_amounts = Vec::new();
    for &index in &contained_indices {
        let event = &senders_deposits[index];
        let deposit_index = state
            .deposit_hash_tree
            .get_index(event.deposit().hash())
            .unwrap();
        if let Some(leaf_index) = state.short_term_eligible_tree.get_leaf_index(deposit_index) {
            let leaf = state
                .short_term_eligible_tree
                .tree
                .get_leaf(leaf_index as usize);
            short_term_eligible_amounts.push(leaf.amount);
            short_term_eligible_indices.push(index as usize);
        }
        if let Some(leaf_index) = state.long_term_eligible_tree.get_leaf_index(deposit_index) {
            let leaf = state
                .long_term_eligible_tree
                .tree
                .get_leaf(leaf_index as usize);
            long_term_eligible_amounts.push(leaf.amount);
            long_term_eligible_indices.push(index as usize);
        }
    }

    let mut short_term_claimed_indices = Vec::new();
    let mut short_term_not_claimed_indices = Vec::new();
    let mut short_term_claimable_amount = U256::zero();

    for &index in &short_term_eligible_indices {
        let event = &senders_deposits[index];
        let salt = derive_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce);
        let nullifier = get_deposit_nullifier(&event.deposit(), salt);
        let is_exists = get_short_term_claim_nullifier_exists(nullifier).await?;
        if is_exists {
            short_term_claimed_indices.push(index);
        } else {
            short_term_not_claimed_indices.push(index);
            let eligible_amount =
                U256::from_big_endian(&short_term_eligible_amounts[index].to_bytes_be());
            short_term_claimable_amount += eligible_amount;
        }
    }

    let mut long_term_claimed_indices = Vec::new();
    let mut long_term_not_claimed_indices = Vec::new();
    let mut long_term_claimable_amount = U256::zero();

    for &index in &long_term_eligible_indices {
        let event = &senders_deposits[index];
        let salt = derive_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce);
        let nullifier = get_deposit_nullifier(&event.deposit(), salt);
        let is_exists = get_long_term_claim_nullifier_exists(nullifier).await?;
        if is_exists {
            long_term_claimed_indices.push(index);
        } else {
            long_term_not_claimed_indices.push(index);
            let eligible_amount =
                U256::from_big_endian(&long_term_eligible_amounts[index].to_bytes_be());
            long_term_claimable_amount += eligible_amount;
        }
    }

    Ok(AssetsStatus {
        senders_deposits,
        contained_indices,
        rejected_indices,
        cancelled_indices,
        pending_indices,
        withdrawn_indices,
        not_withdrawn_indices,
        short_term_eligible_indices,
        short_term_claimed_indices,
        short_term_not_claimed_indices,
        short_term_claimable_amount,
        long_term_eligible_indices,
        long_term_claimed_indices,
        long_term_not_claimed_indices,
        long_term_claimable_amount,
    })
}

impl AssetsStatus {
    pub fn get_not_withdrawn_events(&self) -> Vec<Deposited> {
        self.not_withdrawn_indices
            .iter()
            .map(|&index| self.senders_deposits[index].clone())
            .collect()
    }

    pub fn get_not_claimed_events(&self) -> Vec<Deposited> {
        self.short_term_not_claimed_indices
            .iter()
            .map(|&index| self.senders_deposits[index].clone())
            .collect()
    }

    pub fn no_remaining(&self) -> bool {
        let income = self.senders_deposits.len();
        let outcome = self.withdrawn_indices.len() + self.cancelled_indices.len();
        income == outcome
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore]
    async fn test_assets_status() {
        let mut state = crate::test::get_dummy_state().await;
        state.sync_trees().await.unwrap();

        let dummy_key = crate::test::get_dummy_keys();

        let result = super::fetch_assets_status(
            &state,
            dummy_key.deposit_address,
            dummy_key.deposit_private_key,
        )
        .await
        .unwrap();
        dbg!(result);
    }
}
