use anyhow::ensure;
use ethers::{
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::Wallet,
    types::{Address, U256},
};

use crate::{
    cli::console::{print_status, print_warning},
    external_api::contracts::utils::get_client,
};

pub async fn handle_contract_call<S: ToString>(
    tx: ethers::contract::builders::ContractCall<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
    from_address: Address,
    from_name: S,
    tx_name: S,
) -> anyhow::Result<()> {
    loop {
        let result = tx.send().await;
        match result {
            Ok(tx) => {
                let pending_tx = tx;
                print_status(format!(
                    "{} tx hash: {:?}",
                    tx_name.to_string(),
                    pending_tx.tx_hash()
                ));
                let tx_receipt = pending_tx.await?.unwrap();
                ensure!(
                    tx_receipt.status.unwrap() == 1.into(),
                    "{} tx failed",
                    from_name.to_string()
                );
                return Ok(());
            }
            Err(e) => {
                let error_message = e.to_string();
                // insufficient balance
                if error_message.contains("-32000") {
                    let estimate_gas = tx.estimate_gas().await?;
                    let gas_price = get_client().await?.get_gas_price().await?;
                    let value = tx.tx.value().cloned().unwrap_or_default();
                    let necessary_balance = estimate_gas * gas_price + value;
                    insuffient_balance_instruction(
                        from_address,
                        necessary_balance,
                        &from_name.to_string(),
                    )
                    .await?;
                    print_status(format!("Retrying {} transaction...", tx_name.to_string()));
                } else {
                    return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
                }
            }
        }
    }
}

async fn insuffient_balance_instruction(
    address: Address,
    required_balance: U256,
    name: &str,
) -> anyhow::Result<()> {
    let client = get_client().await?;
    let balance = client.get_balance(address, None).await?;
    print_warning(format!(
        r"Insufficient balance of {} address {:?}. 
Current balance: {} ETH. At least {} ETH is required for the transaction.
Waiting for your deposit...",
        name,
        address,
        pretty_format_u256(balance),
        pretty_format_u256(required_balance)
    ));
    loop {
        let client = get_client().await?;
        let new_balance = client.get_balance(address, None).await?;
        if new_balance > balance {
            print_status("Balance updated");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    Ok(())
}

pub fn pretty_format_u256(value: U256) -> String {
    let s = ethers::utils::format_units(value, "ether").unwrap();
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pretty_format() {
        let value = ethers::utils::parse_ether("1.01000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1.01");

        let value = ethers::utils::parse_ether("1.00000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1");
    }
}
