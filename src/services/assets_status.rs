use anyhow::ensure;
use ethers::types::{Address, H256};
use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u256::U256,
    utils::leafable::Leafable as _,
};
use mining_circuit_v1::claim::claim_inner_circuit::get_deposit_nullifier;

use crate::{
    external_api::contracts::{
        events::{get_deposited_event_by_sender, Deposited},
        int1::{
            get_deposit_data, get_last_processed_deposit_id, get_withdrawal_nullifier_exists,
            DepositData,
        },
        minter::get_claim_nullifier_exists,
    },
    utils::{
        deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap,
        salt::get_salt_from_private_key_nonce,
    },
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
    pub eligible_indices: Vec<usize>,      // Positions in senders_deposits that are eligible
    pub claimed_indices: Vec<usize>,       // Positions in senders_deposits that are claimed
    pub not_claimed_indices: Vec<usize>, // Positions in senders_deposits that are eligible but not claimed
}

pub async fn fetch_assets_status(
    deposit_hash_tree: &DepositHashTree,
    eligible_tree: &EligibleTreeWithMap,
    deposit_address: Address,
    deposit_private_key: H256,
) -> anyhow::Result<AssetsStatus> {
    let senders_deposits = get_deposited_event_by_sender(deposit_address).await?;

    let mut contained_indices = Vec::new();
    let mut not_contained_indices = Vec::new();
    for (index, event) in senders_deposits.iter().enumerate() {
        if deposit_hash_tree
            .get_index(event.deposit().hash())
            .is_some()
        {
            contained_indices.push(index);
        } else {
            not_contained_indices.push(index);
        }
    }

    let last_processed_deposit_id = get_last_processed_deposit_id().await?;
    let mut rejected_indices = Vec::new();
    let mut cancelled_indices = Vec::new();
    let mut pending_indices = Vec::new();
    for &index in &not_contained_indices {
        let event = &senders_deposits[index];
        let deposit_data = get_deposit_data(event.deposit_id).await?;
        if deposit_data.is_rejected {
            ensure!(
                event.deposit_id <= last_processed_deposit_id,
                "Error Inconsistency: Rejected deposit ID is not processed yet"
            );
            rejected_indices.push(index);
        } else if deposit_data == DepositData::default() {
            cancelled_indices.push(index);
        } else {
            ensure!(
                last_processed_deposit_id < event.deposit_id,
                "Error Inconsistency: Pending deposit ID is processed"
            );
            pending_indices.push(index);
        }
    }

    let mut withdrawn_indices = Vec::new();
    let mut not_withdrawn_indices = Vec::new();
    for &index in contained_indices.iter() {
        let event = &senders_deposits[index];
        let salt = get_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce);
        let nullifier = get_pubkey_salt_hash(U256::default(), salt);
        let is_exists = get_withdrawal_nullifier_exists(nullifier).await?;
        if is_exists {
            withdrawn_indices.push(index);
        } else {
            not_withdrawn_indices.push(index);
        }
    }

    let mut eligible_indices = Vec::new();
    for &index in &contained_indices {
        let event = &senders_deposits[index];
        let deposit_index = deposit_hash_tree.get_index(event.deposit().hash()).unwrap();
        if eligible_tree.get_leaf_index(deposit_index).is_some() {
            eligible_indices.push(index);
        }
    }

    let mut claimed_indices = Vec::new();
    let mut not_claimed_indices = Vec::new();
    for &index in &eligible_indices {
        let event = &senders_deposits[index];
        let salt = get_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce);
        let nullifier = get_deposit_nullifier(&event.deposit(), salt);
        let is_exists = get_claim_nullifier_exists(nullifier).await?;
        if is_exists {
            claimed_indices.push(index);
        } else {
            not_claimed_indices.push(index);
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
        eligible_indices,
        claimed_indices,
        not_claimed_indices,
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
        self.not_claimed_indices
            .iter()
            .map(|&index| self.senders_deposits[index].clone())
            .collect()
    }
}

// #[cfg(test)]
// mod tests {
//     #[tokio::test]
//     async fn test_assets_status() {
//         let mut state = crate::test::get_dummy_state().await;
//         state.sync_trees().await.unwrap();

//         let result = super::fetch_assets_status(
//             &state.deposit_hash_tree,
//             &state.eligible_tree,
//             key.deposit_address,
//             key.deposit_private_key,
//         )
//         .await
//         .unwrap();
//         dbg!(result);
//     }
// }
