use super::{error::BlockchainError, utils::ProviderWithSigner};
use alloy::{
    consensus::{Transaction as _, TxEip1559},
    primitives::TxHash,
    providers::{PendingTransactionError, Provider as _},
    rpc::types::TransactionRequest,
};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(20);
const MAX_GAS_BUMP_ATTEMPTS: u32 = 3;
const GAS_BUMP_PERCENTAGE: u64 = 25; // Should be above 10 to avoid replacement transaction underpriced error

pub async fn send_transaction_with_gas_bump(
    signer: ProviderWithSigner,
    tx_request: TransactionRequest,
    tx_name: &str,
) -> Result<TxHash, BlockchainError> {
    let sendable_tx = signer.fill(tx_request).await?;
    let tx_envelope = sendable_tx.try_into_envelope().unwrap();
    let tx_hash = *tx_envelope.hash();
    // make tx eip1559 object to get parameters
    let tx_eip1559 = tx_envelope.as_eip1559().unwrap().tx().clone();
    log::info!(
        "Sending transaction: {} with nonce {}, gas limit {}, value {}, max fee per gas {:?}, max priority fee per gas {:?}",
        tx_name,
        tx_eip1559.nonce,
        tx_eip1559.gas_limit,
        tx_eip1559.value,
        tx_eip1559.max_fee_per_gas,
        tx_eip1559.max_priority_fee_per_gas
    );
    match signer
        .send_tx_envelope(tx_envelope)
        .await?
        .with_timeout(Some(TIMEOUT))
        .watch()
        .await
    {
        Ok(tx_hash) => {
            log::info!(
                "Transaction sent: {:?} with tx hash: {:?}",
                tx_name.to_string(),
                tx_hash
            );
            Ok(tx_hash)
        }
        Err(PendingTransactionError::TxWatcher(_)) => {
            // timeout, so we need to bump the gas
            resend_tx_with_gas_bump(signer, tx_hash, &tx_eip1559, tx_name).await
        }
        Err(e) => Err(BlockchainError::TransactionError(format!(
            "{tx_name} failed with error: {e:?}"
        ))),
    }
}

async fn resend_tx_with_gas_bump(
    signer: ProviderWithSigner,
    initial_tx_hash: TxHash,
    tx_eip1559: &TxEip1559,
    tx_name: &str,
) -> Result<TxHash, BlockchainError> {
    log::info!("Resending transaction: {tx_name}");
    let mut pending_tx_hashes = vec![initial_tx_hash];

    let mut current_tx = tx_eip1559.clone();

    for attempt in 1..=MAX_GAS_BUMP_ATTEMPTS {
        // check if previous tx succeeded
        for tx_hash in pending_tx_hashes.iter().rev() {
            let tx_receipt = signer.get_transaction_receipt(*tx_hash).await?;
            if let Some(tx_receipt) = tx_receipt {
                log::info!(
                    "Previous tx settled with hash: {:?}",
                    tx_receipt.transaction_hash
                );
                if tx_receipt.status() {
                    return Ok(tx_receipt.transaction_hash);
                } else {
                    return Err(BlockchainError::TransactionFailed(format!(
                        "Transaction {} failed with tx hash: {:?}",
                        tx_name, tx_receipt.transaction_hash
                    )));
                }
            }
        }
        // bump gas
        let fee_estimation = signer.estimate_eip1559_fees().await?;

        let (new_max_priority_fee_per_gas, new_max_fee_per_gas) =
            if fee_estimation.max_priority_fee_per_gas > current_tx.max_priority_fee_per_gas {
                // use the estimated fee which is higher than the current fee
                (
                    fee_estimation.max_priority_fee_per_gas,
                    fee_estimation.max_fee_per_gas,
                )
            } else {
                // bump the gas by a percentage
                (
                    current_tx.max_priority_fee_per_gas * (100 + GAS_BUMP_PERCENTAGE as u128) / 100,
                    current_tx.max_fee_per_gas * (100 + GAS_BUMP_PERCENTAGE as u128) / 100,
                )
            };

        let new_tx_request = TransactionRequest::default()
            .max_priority_fee_per_gas(new_max_priority_fee_per_gas)
            .max_fee_per_gas(new_max_fee_per_gas)
            .nonce(current_tx.nonce)
            .to(current_tx.to().unwrap())
            .nonce(current_tx.nonce)
            .gas_limit(current_tx.gas_limit)
            .input(current_tx.input.into())
            .value(current_tx.value);

        // send the new transaction
        let sendable_tx = signer.fill(new_tx_request).await?;
        let tx_envelope = sendable_tx.try_into_envelope().unwrap();
        log::info!(
            "Sending bumped gas tx {tx_name} attempt: {attempt} with new max_fee_per_gas: {new_max_fee_per_gas:?}, new max_priority_fee_per_gas: {new_max_priority_fee_per_gas:?}",
        );

        match signer
            .send_tx_envelope(tx_envelope.clone())
            .await?
            .with_timeout(Some(TIMEOUT))
            .watch()
            .await
        {
            Ok(tx_hash) => {
                println!("Transaction sent: {tx_hash:?}");
                return Ok(tx_hash);
            }
            Err(PendingTransactionError::TxWatcher(_)) => {
                // timeout, so we need to bump the gas again
                log::info!("Transaction timed out, bumping gas again");
            }
            Err(e) => {
                return Err(BlockchainError::TransactionError(format!(
                    "{tx_name} failed with error: {e:?}"
                )));
            }
        }
        // update the current transaction
        current_tx = tx_envelope.as_eip1559().unwrap().tx().clone();
        pending_tx_hashes.push(*tx_envelope.tx_hash());
    }
    Err(BlockchainError::MaxTxRetriesReached)
}
