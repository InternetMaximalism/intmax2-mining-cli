use crate::{
    external_api::contracts::utils::get_wallet,
    state::private_data::{load_encrypted_private_data, write_encrypted_private_data, PrivateData},
    utils::network::get_network,
};
use dialoguer::{Input, Password};
use ethers::signers::Signer;
use ethers::types::{H160, H256};

/// Require input or load private data
pub async fn set_private_data() -> anyhow::Result<PrivateData> {
    match load_encrypted_private_data() {
        Some(input) => {
            let password: String = Password::new().with_prompt("Password").interact()?;
            let private_data = PrivateData::decrypt(&password, &input)?;
            Ok(private_data)
        }
        None => {
            let deposit_private_key_str: String = Password::new()
                .with_prompt(format!(
                    "Private key of deposit account on {}",
                    get_network(),
                ))
                .validate_with(|input: &String| validate_private_key(input))
                .interact()?;

            let deposit_private_key = deposit_private_key_str.parse()?;
            let deposit_address = get_wallet(deposit_private_key).await?.address();
            println!("Deposit address: {:?}", deposit_address);

            let claim_private_key_str: String = Password::new()
                .with_prompt(format!("Private key of claim account on {}", get_network(),))
                .validate_with(|input: &String| validate_private_key(input))
                .interact()?;
            let claim_private_key = claim_private_key_str.parse()?;
            let claim_address = get_wallet(claim_private_key).await?.address();
            println!("Claim address: {:?}", claim_address);

            let withdrawal_address_str: String = Input::new()
                .with_prompt(format!(
                    "Address of withdrawal account on {}",
                    get_network(),
                ))
                .validate_with(|input: &String| validate_address(input))
                .interact()?;
            let private_data = PrivateData::new(
                &deposit_private_key_str,
                &claim_private_key_str,
                &withdrawal_address_str,
            )
            .await?;

            if private_data.deposit_address == private_data.withdrawal_address {
                return Err(anyhow::anyhow!(
                    "Deposit and withdrawal addresses must be different"
                ));
            }
            if private_data.deposit_address == private_data.claim_address {
                return Err(anyhow::anyhow!(
                    "Deposit and claim addresses must be different"
                ));
            }
            if private_data.claim_address == private_data.withdrawal_address {
                return Err(anyhow::anyhow!(
                    "Claim and withdrawal addresses must be different"
                ));
            }

            let password = Password::new()
                .with_prompt("Password to encrypt private key")
                .with_confirmation("Confirm password", "Passwords do not match")
                .interact()?;
            let encrypted_private_data = private_data.encrypt(&password)?;
            // write to file
            write_encrypted_private_data(&encrypted_private_data)?;
            Ok(private_data)
        }
    }
}

fn validate_private_key(input: &str) -> Result<(), &'static str> {
    match input.parse() {
        Ok(H256(x)) => {
            if x == H256::zero().0 {
                return Err("Invalid private key");
            }
            Ok(())
        }
        Err(_) => return Err("Invalid private key"),
    }
}

fn validate_address(input: &str) -> Result<(), &'static str> {
    match input.parse() {
        Ok(H160(x)) => {
            if x == H160::zero().0 {
                return Err("Invalid address");
            }
            Ok(())
        }
        Err(_) => return Err("Invalid address"),
    }
}
