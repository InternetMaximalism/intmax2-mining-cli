use anyhow::{ensure, Ok};
use intmax2_zkp::ethereum_types::{
    address::Address, bytes32::Bytes32, u32limb_trait::U32LimbTrait,
};
use mining_circuit::claim::claim_inner_circuit::ClaimInnerValue;

use crate::{
    external_api::contracts::events::Deposited,
    services::claim::determin::MAX_CLAIMS,
    state::state::State,
    utils::salt::{get_pubkey_from_private_key, get_salt_from_private_key_nonce},
};

pub async fn generate_claim_witness(
    state: &State,
    events: &[Deposited],
) -> anyhow::Result<Vec<ClaimInnerValue>> {
    ensure!(events.len() > 0, "No event to generate witness");
    ensure!(
        events.len() <= MAX_CLAIMS,
        format!("Max {} events to generate witness", MAX_CLAIMS)
    );
    let deposit_tree_root = state.deposit_hash_tree.get_root();
    let eligible_tree_root: Bytes32 = state.eligible_tree.get_root().into();
    let pubkey = get_pubkey_from_private_key(state.private_data.deposit_key);
    let recipient = Address::from_bytes_be(
        &state
            .private_data
            .to_addresses()
            .await?
            .claim_address
            .as_bytes(),
    );
    let mut witnesses = Vec::new();
    let mut prev_claim_hash = Bytes32::default();
    for event in events {
        let deposit_index = event.deposit_index.unwrap();
        let deposit_merkle_proof = state.deposit_hash_tree.prove(deposit_index);
        let deposit = event.deposit();
        let eligible_index = state.eligible_tree.get_leaf_index(deposit_index).unwrap();
        let eligible_merkle_proof = state.eligible_tree.tree.prove(eligible_index as usize);
        let eligible_leaf = state.eligible_tree.tree.get_leaf(eligible_index as usize);
        let salt = get_salt_from_private_key_nonce(
            state.private_data.deposit_key,
            event.tx_nonce.unwrap(),
        );
        let value = ClaimInnerValue::new(
            deposit_tree_root,
            deposit_index,
            deposit_merkle_proof,
            deposit,
            eligible_tree_root,
            eligible_index,
            eligible_merkle_proof,
            eligible_leaf,
            pubkey,
            salt,
            recipient,
            prev_claim_hash,
        )?;
        prev_claim_hash = value.new_claim_hash;
        witnesses.push(value);
    }
    Ok(witnesses)
}
