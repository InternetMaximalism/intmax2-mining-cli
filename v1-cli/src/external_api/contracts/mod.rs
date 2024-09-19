pub mod events;
pub mod int1;
pub mod minter;
pub mod utils;

#[cfg(test)]
mod tests {
    use ethers::{
        providers::Middleware as _,
        signers::Signer,
        types::{Address, TransactionRequest, U256},
    };

    use crate::{
        cli::console::insuffient_balance_instruction,
        external_api::contracts::utils::{get_client_with_signer, get_wallet},
        test::get_dummy_state,
    };

    #[tokio::test]
    async fn test_innsufficient_balance() -> anyhow::Result<()> {
        let state = get_dummy_state();
        let to = "0x0000000000000000000000000000000000000000"
            .parse::<Address>()
            .unwrap();
        let tx = TransactionRequest::new()
            .to(to)
            .value(U256::from(1_000_000_000_000_000_000_000_000u128)); // 1 ETH
        let client = get_client_with_signer(state.private_data.deposit_key).await?;

        let pending_tx = client.send_transaction(tx, None).await;

        match pending_tx {
            Ok(_) => println!("Transaction sent successfully"),
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("-32000") {
                    // Insufficient funds code
                    let address = get_wallet(state.private_data.deposit_key).await?.address();
                    insuffient_balance_instruction(address, "deposit").await?;
                } else {
                    println!("JSON-RPC error: {}", error_message);
                }
            }
        }
        Ok(())
    }
}
