use ethers::types::U256;
use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u32limb_trait::U32LimbTrait as _,
};

use crate::{
    config::{MiningAmount, UserSettings},
    external_api::contracts::{int1::get_int1_contract_with_signer, utils::get_account_nonce},
    services::contracts::handle_contract_call,
    state::state::State,
    utils::salt::{get_pubkey_from_private_key, get_salt_from_private_key_nonce},
};

pub async fn deposit_task(state: &State) -> anyhow::Result<()> {
    let deposit_address = state.private_data.deposit_address;
    let nonce = get_account_nonce(deposit_address).await?;
    let salt = get_salt_from_private_key_nonce(state.private_data.deposit_private_key, nonce);
    let pubkey = get_pubkey_from_private_key(state.private_data.deposit_private_key);
    let pubkey_salt_hash: [u8; 32] = get_pubkey_salt_hash(pubkey, salt)
        .to_bytes_be()
        .try_into()
        .unwrap();

    let mining_amount: U256 = match UserSettings::new()?.mining_amount {
        MiningAmount::OneTenth => ethers::utils::parse_units("0.1", "ether").unwrap().into(),
        MiningAmount::One => ethers::utils::parse_units("1", "ether").unwrap().into(),
    };

    let deposit_address = state.private_data.deposit_address;
    let int1 = get_int1_contract_with_signer(state.private_data.deposit_private_key).await?;
    let mut tx = int1
        .deposit_native_token(pubkey_salt_hash)
        .value(mining_amount);
    tx.tx.set_nonce(nonce);

    handle_contract_call(tx, deposit_address, "deposit", "deposit").await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test::get_dummy_state;

    #[tokio::test]
    async fn test_deposit() {
        let state = get_dummy_state().await;
        super::deposit_task(&state).await.unwrap();
    }
}
