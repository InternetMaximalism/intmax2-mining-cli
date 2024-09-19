use anyhow::ensure;
use ethers::providers::{Http, PendingTransaction};
use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait as _;

use crate::{
    cli::console::{insuffient_balance_instruction, print_status},
    external_api::contracts::{
        events::Deposited,
        int1::{get_int1_contract_with_signer, int_1},
    },
    state::state::State,
};

pub async fn cancel_task(state: &State, event: Deposited) -> anyhow::Result<()> {
    let deposit = int_1::Deposit {
        recipient_salt_hash: event.recipient_salt_hash.to_bytes_be().try_into().unwrap(),
        token_index: event.token_index,
        amount: ethers::types::U256::from_big_endian(&event.amount.to_bytes_be()),
    };
    let deposit_address = state.private_data.to_addresses().await?.deposit_address;

    loop {
        let int1 = get_int1_contract_with_signer(state.private_data.deposit_key).await?;
        let tx = int1.cancel_deposit(event.deposit_id.into(), deposit.clone());
        let result = tx.send().await;
        match result {
            Ok(tx) => {
                let pending_tx: PendingTransaction<Http> = tx;
                print_status(&format!("Cancel tx hash: {:?}", pending_tx.tx_hash()));
                let tx_receipt = pending_tx.await?.unwrap();
                ensure!(tx_receipt.status.unwrap() == 1.into(), "Cancel tx failed");
                return Ok(());
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("-32000") {
                    insuffient_balance_instruction(deposit_address, "deposit").await?;
                    print_status("Retrying cancel transaction...");
                } else {
                    return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
                }
            }
        }
    }
}
