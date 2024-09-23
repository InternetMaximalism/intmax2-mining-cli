use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};
use std::env;
use std::str::FromStr;

use crate::state::keys::{ClaimKeys, Keys, MiningKeys};
use crate::state::mode::RunMode;
use crate::utils::config::Settings;
use crate::utils::errors::CLIError;

pub struct MiningConfig {
    keys: MiningKeys,
    mining_unit: U256,
    max_deposits: usize,
}

pub async fn load_env(mode: RunMode) -> anyhow::Result<Keys> {
    let keys = match mode {
        RunMode::Mining => {
            let deposit_private_keys = load_deposit_private_keys()?;
            let withdrawal_address = load_withdrawal_address()?;
            Keys::Mining(MiningKeys::new(deposit_private_keys, withdrawal_address).await)
        }
        RunMode::Claim => {
            let deposit_private_keys = load_deposit_private_keys()?;
            let claim_private_key = load_claim_private_key()?;
            Keys::Claim(ClaimKeys::new(deposit_private_keys, claim_private_key).await)
        }
        RunMode::Exit => {
            let deposit_private_keys = load_deposit_private_keys()?;
            let withdrawal_address = load_withdrawal_address()?;
            Keys::Mining(MiningKeys::new(deposit_private_keys, withdrawal_address).await)
        }
    };
    Ok(keys)
}

async fn load_rpc_url() -> anyhow::Result<String> {
    let rpc_url = env::var("RPC_URL")
        .map_err(|_| CLIError::EnvError("RPC_URL environment variable is not set".to_string()))?;
    check_rpc_url(&rpc_url)
        .await
        .map_err(|_| CLIError::EnvError("Wrong RPC URL".to_string()))?;
    Ok(rpc_url)
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

fn load_withdrawal_address() -> anyhow::Result<Address> {
    let withdrawal_address = env::var("WITHDRAWAL_ADDRESS").map_err(|_| {
        CLIError::EnvError("WITHDRAWAL_ADDRESS environment variable is not set".to_string())
    })?;
    let address = Address::from_str(&withdrawal_address)
        .map_err(|_| CLIError::EnvError(format!("Invalid address: {}", withdrawal_address)))?;
    Ok(address)
}

async fn check_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let client = ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)?;
    let chain_id = client.get_chainid().await?;
    let setting = Settings::new()?;
    if chain_id != setting.blockchain.chain_id.into() {
        anyhow::bail!(
            "RPC URL chain id {} does not match the expected chain id {}",
            chain_id.as_u64(),
            setting.blockchain.chain_id,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_deposit_private_keys() {
        let keys = super::load_deposit_private_keys().unwrap();
        dbg!(&keys);
    }
}
