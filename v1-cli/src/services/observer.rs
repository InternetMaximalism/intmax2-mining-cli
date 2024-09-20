use ethers::types::{Address, H256};
use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u256::U256,
    utils::leafable::Leafable as _,
};

use crate::{
    external_api::contracts::{
        events::{get_deposited_event, DepositQuery, Deposited},
        int1::{get_deposit_data, get_withdrawal_nullifier_exists, DepositData},
    },
    utils::{deposit_hash_tree::DepositHashTree, salt::get_salt_from_private_key_nonce},
};

#[derive(Debug, Clone)]
pub struct AssetsStatus {
    pub senders_deposits: Vec<Deposited>,
    pub deposit_indices: Vec<Option<u32>>, // Deposit indices of senders_deposits
    pub contained_indices: Vec<usize>, // Positions in senders_deposits that are contained in the deposit tree
    pub not_contained_indices: Vec<usize>, // Positions in senders_deposits that are not contained in the deposit tree
    pub rejected_indices: Vec<usize>,      // Positions in senders_deposits that are rejected
    pub cancelled_indices: Vec<usize>,     // Positions in senders_deposits that are cancelled
    pub pending_indices: Vec<usize>, // Positions in senders_deposits that are not analyzed yet
    pub withdrawn_indices: Vec<usize>, // Positions in senders_deposits that are withdrawn
    pub not_withdrawn_indices: Vec<usize>, // Positions in senders_deposits that are deposited but not withdrawn
}

pub async fn observe_assets_status(
    deposit_hash_tree: &DepositHashTree,
    deposit_address: Address,
    deposit_private_key: H256,
) -> anyhow::Result<AssetsStatus> {
    let senders_deposits = get_deposited_event(DepositQuery::BySender(deposit_address)).await?;

    let mut contained_indices = Vec::new();
    let mut not_contained_indices = Vec::new();
    let mut deposit_indices = Vec::new();

    for (index, event) in senders_deposits.iter().enumerate() {
        let deposit_index = deposit_hash_tree.get_index(event.deposit().hash());
        deposit_indices.push(deposit_index);
        if deposit_index.is_some() {
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
            pending_indices.push(index);
        }
    }

    let mut withdrawn_indices = Vec::new();
    let mut not_withdrawn_indices = Vec::new();
    for &index in contained_indices.iter() {
        let event = &senders_deposits[index];
        let salt = get_salt_from_private_key_nonce(deposit_private_key, event.tx_nonce.unwrap());
        let nullifier = get_pubkey_salt_hash(U256::default(), salt);
        let is_exists = get_withdrawal_nullifier_exists(nullifier).await?;
        if is_exists {
            withdrawn_indices.push(index);
        } else {
            not_withdrawn_indices.push(index);
        }
    }

    Ok(AssetsStatus {
        senders_deposits,
        deposit_indices,
        contained_indices,
        not_contained_indices,
        rejected_indices,
        cancelled_indices,
        pending_indices,
        withdrawn_indices,
        not_withdrawn_indices,
    })
}

impl AssetsStatus {}
