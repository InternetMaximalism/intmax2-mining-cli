use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait as _;

use crate::{
    cli::console::print_log,
    external_api::contracts::{
        events::Deposited,
        int1::{get_int1_contract_with_signer, int_1},
    },
    services::utils::{await_until_low_gas_price, handle_contract_call, set_gas_price},
    state::{key::Key, state::State},
};

pub async fn cancel_task(_state: &State, key: &Key, event: Deposited) -> anyhow::Result<()> {
    let deposit = int_1::Deposit {
        recipient_salt_hash: event.recipient_salt_hash.to_bytes_be().try_into().unwrap(),
        token_index: event.token_index,
        amount: ethers::types::U256::from_big_endian(&event.amount.to_bytes_be()),
    };
    let deposit_address = key.deposit_address;

    await_until_low_gas_price().await?;
    let int1 = get_int1_contract_with_signer(key.deposit_private_key).await?;
    let mut tx = int1.cancel_deposit(event.deposit_id.into(), deposit.clone());
    set_gas_price(&mut tx).await?;
    let tx_hash = handle_contract_call(tx, deposit_address, "deposit", "cancel").await?;
    print_log(format!(
        "Cancelled deposit id {:?} with tx hash {:?}",
        event.deposit_id, tx_hash,
    ));
    Ok(())
}
