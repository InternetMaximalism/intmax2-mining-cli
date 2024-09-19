use anyhow::ensure;
use ethers::{
    providers::{Http, PendingTransaction},
    types::U256,
};
use intmax2_zkp::{
    common::deposit::get_pubkey_salt_hash, ethereum_types::u32limb_trait::U32LimbTrait as _,
};

use crate::{
    cli::console::{insuffient_balance_instruction, print_status},
    config::{MiningAmount, UserSettings},
    external_api::contracts::{int1::get_int1_contract_with_signer, utils::get_account_nonce},
    state::state::State,
    utils::salt::{get_pubkey_from_private_key, get_salt_from_private_key_nonce},
};

pub async fn deposit_task(state: &State) -> anyhow::Result<()> {
    let deposit_address = state.private_data.to_addresses().await?.deposit_address;
    let nonce = get_account_nonce(deposit_address).await?;
    let salt = get_salt_from_private_key_nonce(state.private_data.deposit_key, nonce);
    let pubkey = get_pubkey_from_private_key(state.private_data.deposit_key);
    let pubkey_salt_hash: [u8; 32] = get_pubkey_salt_hash(pubkey, salt)
        .to_bytes_be()
        .try_into()
        .unwrap();

    let mining_amount: U256 = match UserSettings::new()?.mining_amount {
        MiningAmount::OneTenth => ethers::utils::parse_units("0.1", "ether").unwrap().into(),
        MiningAmount::One => ethers::utils::parse_units("1", "ether").unwrap().into(),
    };

    let deposit_address = state.private_data.to_addresses().await?.deposit_address;

    loop {
        let int1 = get_int1_contract_with_signer(state.private_data.deposit_key).await?;
        let tx = int1
            .deposit_native_token(pubkey_salt_hash)
            .value(mining_amount);
        let result = tx.send().await;
        match result {
            Ok(tx) => {
                let pending_tx: PendingTransaction<Http> = tx;
                print_status(&format!("Deposit tx hash: {:?}", pending_tx.tx_hash()));
                let tx_receipt = pending_tx.await?.unwrap();
                ensure!(tx_receipt.status.unwrap() == 1.into(), "Deposit tx failed");

                // reduce remaining deposits
                let mut user_settings = UserSettings::new()?;
                user_settings.remaining_deposits -= 1;
                user_settings.save()?;
                return Ok(());
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("-32000") {
                    insuffient_balance_instruction(deposit_address, "deposit").await?;
                    print_status("Retrying deposit transaction...");
                } else {
                    return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::get_dummy_state;

    #[tokio::test]
    async fn test_deposit() {
        let state = get_dummy_state();
        super::deposit_task(&state).await.unwrap();
    }
}
