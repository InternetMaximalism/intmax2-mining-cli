use alloy::{
    primitives::{Address, B256, U256},
    providers::Provider as _,
    rpc::types::TransactionRequest,
};

use crate::{
    cli::console::print_warning,
    external_api::contracts::{
        error::BlockchainError,
        handlers::send_transaction_with_gas_bump,
        utils::{get_address_from_private_key, get_provider_with_signer, NormalProvider},
    },
};

pub async fn balance_transfer(
    provider: &NormalProvider,
    deposit_private_key: B256,
    to_address: Address,
) -> Result<(), BlockchainError> {
    let signer = get_provider_with_signer(provider, deposit_private_key);
    let deposit_address = get_address_from_private_key(deposit_private_key);
    let balance = provider.get_balance(deposit_address).await?;
    // todo: use estimate gas
    let gas_limit = U256::from(10000);
    let gas_price = U256::from(provider.get_gas_price().await?);

    if balance < gas_price * gas_limit {
        print_warning("Insufficient balance to transfer");
        return Ok(());
    }
    let transfer_amount = balance - gas_price * gas_limit;
    let tx_request = TransactionRequest::default()
        .to(to_address)
        .value(transfer_amount);
    send_transaction_with_gas_bump(signer, tx_request, "send balance").await?;
    Ok(())
}
