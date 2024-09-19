use crate::private_data::{load_encrypted_private_data, write_encrypted_private_data, PrivateData};
use dialoguer::{Input, Password};
use ethers::types::{H160, H256};

/// Require input or load private data
pub fn set_private_data() -> anyhow::Result<PrivateData> {
    match load_encrypted_private_data() {
        Some(input) => {
            let password: String = Password::new().with_prompt("Password").interact()?;
            let private_data = PrivateData::decrypt(&password, &input)?;
            Ok(private_data)
        }
        None => {
            let deposit_private_key: String = Password::new()
                .with_prompt("Deposit private key")
                .validate_with(|input: &String| validate_private_key(input))
                .interact()?;
            let claim_private_key: String = Password::new()
                .with_prompt("Claim private key")
                .validate_with(|input: &String| validate_private_key(input))
                .interact()?;
            let withdrawal_address: String = Input::new()
                .with_prompt("Withdrawal address")
                .validate_with(|input: &String| validate_address(input))
                .interact()?;
            let private_data = PrivateData::new(
                &deposit_private_key,
                &claim_private_key,
                &withdrawal_address,
            )?;
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
        Ok(H256(_)) => Ok(()),
        Err(_) => return Err("Invalid private key"),
    }
}

fn validate_address(input: &str) -> Result<(), &'static str> {
    match input.parse() {
        Ok(H160(_)) => Ok(()),
        Err(_) => return Err("Invalid address"),
    }
}
