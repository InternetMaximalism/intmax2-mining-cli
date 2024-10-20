use ethers::{
    providers::Middleware,
    types::{Address, Eip1559TransactionRequest, H256},
};
use log::info;

use crate::{
    cli::console::{print_status, print_warning},
    external_api::contracts::{
        error::BlockchainError,
        utils::{
            get_address, get_balance, get_client_with_signer, get_eip1559_fees, get_tx_receipt,
        },
    },
    utils::retry::with_retry,
};

pub async fn balance_transfer(
    deposit_private_key: H256,
    to_address: Address,
) -> Result<(), BlockchainError> {
    let deposit_address = get_address(deposit_private_key);
    let balance = get_balance(deposit_address).await?;
    let client = get_client_with_signer(deposit_private_key).await?;
    let gas_limit = {
        with_retry(|| async {
            let tx = Eip1559TransactionRequest::new()
                .to(to_address)
                .value(balance);
            client.estimate_gas(&tx.into(), None).await
        })
        .await
        .map_err(|_| BlockchainError::NetworkError("Failed to estimate gas".to_string()))?
    };
    info!("Estimated gas limit: {}", gas_limit);
    let (max_gas_price, max_priority_fee_per_gas) = get_eip1559_fees().await?;
    let gas_price = max_gas_price + max_priority_fee_per_gas;

    if balance < gas_price * gas_limit {
        print_warning("Insufficient balance to transfer");
        return Ok(());
    }
    let transfer_amount = balance - gas_price * gas_limit;
    let signer = get_client_with_signer(deposit_private_key).await?;
    let tx = Eip1559TransactionRequest::new()
        .to(to_address)
        .value(transfer_amount)
        .max_fee_per_gas(max_gas_price)
        .max_priority_fee_per_gas(max_priority_fee_per_gas);
    let pending_tx = signer
        .send_transaction(tx, None)
        .await
        .map_err(|_| BlockchainError::NetworkError("Failed to send transaction".to_string()))?;
    print_status(format!("Transfer tx hash: {:?}", pending_tx.tx_hash()));
    let reciept = get_tx_receipt(pending_tx.tx_hash()).await?;
    if reciept.status.unwrap() != 1.into() {
        print_warning("Transfer failed. Please retry.");
    }
    print_status("Transfer successful");
    Ok(())
}
