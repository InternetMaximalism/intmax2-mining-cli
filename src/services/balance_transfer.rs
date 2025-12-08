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

    // Estimate gas and fees to avoid under-budgeting and leave room for gas bumps.
    let estimate_request = TransactionRequest::default().to(to_address);
    let estimated_gas = provider.estimate_gas(estimate_request).await?;
    let fee_estimation = signer.estimate_eip1559_fees().await?;
    let gas_limit = U256::from(estimated_gas);
    let max_fee_per_gas = U256::from(fee_estimation.max_fee_per_gas);
    let gas_budget = gas_limit * max_fee_per_gas * U256::from(2u64);

    if balance <= gas_budget {
        print_warning("Insufficient balance to transfer");
        return Ok(());
    }
    let transfer_amount = balance - gas_budget;
    let tx_request = TransactionRequest::default()
        .to(to_address)
        .value(transfer_amount)
        .gas_limit(estimated_gas)
        .max_fee_per_gas(fee_estimation.max_fee_per_gas)
        .max_priority_fee_per_gas(fee_estimation.max_priority_fee_per_gas);
    send_transaction_with_gas_bump(
        provider,
        signer,
        tx_request,
        "send balance",
        "deposit address",
    )
    .await?;
    Ok(())
}
