use ethers::types::H256;
use std::env;
use std::str::FromStr;

use crate::state::mode::RunMode;
use crate::utils::errors::CLIError;

pub fn load_env(mode: RunMode) {
    if mode == RunMode::Claim {
        
    }
}

fn load_deposit_private_keys() -> anyhow::Result<Vec<H256>> {
    let deposit_private_keys =
        env::var("DEPOSIT_PRIVATE_KEYS").unwrap_or_else(|_| "[]".to_string());
    let parsed: Vec<String> = serde_json::from_str(&deposit_private_keys).map_err(|_| {
        CLIError::EnvError("Invalid DEPOSIT_PRIVATE_KEYS environment variable. Please set as format of DEPOSIT_PRIVATE_KEYS='[\"0xa...\", \"0xb...\"]'".to_string())
    })?;
    let keys = parsed
        .iter()
        .map(|key| {
            H256::from_str(key)
                .map_err(|_| CLIError::EnvError(format!("Invalid private key: {}", key)))
        })
        .collect::<Result<Vec<H256>, CLIError>>()?;
    Ok(keys)
}

fn load_claim_private_key() -> anyhow::Result<H256> {
    let claim_private_key = env::var("CLAIM_PRIVATE_KEY").map_err(|_| {
        CLIError::EnvError("CLAIM_PRIVATE_KEY environment variable is not set".to_string())
    })?;
    let key = H256::from_str(&claim_private_key)
        .map_err(|_| CLIError::EnvError(format!("Invalid private key: {}", claim_private_key)))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_deposit_private_keys() {
        let keys = super::load_deposit_private_keys().unwrap();
        dbg!(&keys);
    }
}
