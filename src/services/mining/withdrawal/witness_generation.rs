use intmax2_zkp::utils::leafable::Leafable as _;
use log::info;
use mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalValue;

use crate::{
    external_api::contracts::{convert::convert_address_to_intmax, events::Deposited},
    state::{key::Key, state::State},
    utils::derive_key::{derive_pubkey_from_private_key, derive_salt_from_private_key_nonce},
};

pub fn generate_withdrawal_witness(
    state: &State,
    key: &Key,
    event: Deposited,
) -> anyhow::Result<SimpleWithdrawalValue> {
    info!("Generating withdrawal witness for {:?}", event);
    let deposit_root = state.deposit_hash_tree.get_root();
    let deposit_index = state
        .deposit_hash_tree
        .get_index(event.deposit().hash())
        .unwrap();
    let deposit_merkle_proof = state.deposit_hash_tree.prove(deposit_index);
    let recipient = convert_address_to_intmax(key.withdrawal_address);
    let pubkey = derive_pubkey_from_private_key(key.deposit_private_key);
    let salt = derive_salt_from_private_key_nonce(key.deposit_private_key, event.tx_nonce);
    let deposit_leaf = event.deposit();
    let withdrawal_value = SimpleWithdrawalValue::new(
        deposit_root,
        deposit_index,
        deposit_leaf,
        deposit_merkle_proof,
        recipient,
        pubkey,
        salt,
    );
    Ok(withdrawal_value)
}
