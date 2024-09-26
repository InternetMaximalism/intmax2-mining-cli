use crate::state::keys::Keys;
use crate::utils::config::Settings;
use crate::utils::errors::CLIError;
use ethers::providers::Middleware;
use ethers::types::{H256, U256};
use std::env;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub keys: Keys,
    pub mining_unit: U256,
    pub mining_times: usize,
}

impl Config {
    pub async fn load() -> anyhow::Result<Self> {
        let _rpc_url = load_rpc_url().await?;
        let _max_gas_price = load_max_gas_price()?;
        let mining_unit = load_mining_unit()?;
        let mining_times = load_mining_times()?;
        let deposit_private_keys = load_deposit_private_keys()?;
        let withdrawal_private_key = load_withdrawal_private_key()?;
        let keys = Keys::new(deposit_private_keys, withdrawal_private_key);
        check_keys(&keys)?;
        Ok(Config {
            keys,
            mining_unit,
            mining_times,
        })
    }
}

fn load_mining_unit() -> anyhow::Result<U256> {
    let mining_unit = env::var("MINING_UNIT").map_err(|_| {
        CLIError::EnvError("MINING_UNIT environment variable is not set".to_string())
    })?;
    if mining_unit != "1".to_string() && mining_unit != "0.1".to_string() {
        return Err(CLIError::EnvError(
            "MINING_UNIT environment variable must be either '1' or '0.1'".to_string(),
        )
        .into());
    }
    let mining_unit = ethers::utils::parse_ether(&mining_unit).unwrap();
    Ok(mining_unit)
}

fn load_mining_times() -> anyhow::Result<usize> {
    let mining_times = env::var("MINING_TIMES").map_err(|_| {
        CLIError::EnvError("MINING_TIMES environment variable is not set".to_string())
    })?;
    let mining_times = mining_times
        .parse::<usize>()
        .map_err(|_| CLIError::EnvError("Invalid MINING_TIMES environment variable".to_string()))?;
    if mining_times != 10 && mining_times != 100 {
        // return CLIError
        return Err(CLIError::EnvError(
            "MINING_TIMES environment variable must be either '10' or '100'".to_string(),
        )
        .into());
    }
    Ok(mining_times)
}

async fn load_rpc_url() -> anyhow::Result<String> {
    let rpc_url = env::var("RPC_URL")
        .map_err(|_| CLIError::EnvError("RPC_URL environment variable is not set".to_string()))?;
    check_rpc_url(&rpc_url)
        .await
        .map_err(|_| CLIError::EnvError(format!("Wrong RPC_URL {}", rpc_url)))?;
    Ok(rpc_url)
}

fn load_deposit_private_keys() -> anyhow::Result<Vec<H256>> {
    let deposit_private_keys = env::var("DEPOSIT_PRIVATE_KEYS").map_err(|_| {
        CLIError::EnvError("DEPOSIT_PRIVATE_KEYS environment variable is not set".to_string())
    })?;
    let parsed: Vec<String> = serde_json::from_str(&deposit_private_keys).map_err(|_| {
        CLIError::EnvError("Invalid DEPOSIT_PRIVATE_KEYS environment variable. Please set as format of DEPOSIT_PRIVATE_KEYS='[\"0xa...\", \"0xb...\"]'".to_string())
    })?;
    let keys = parsed
        .iter()
        .map(|key| {
            H256::from_str(key).map_err(|_| {
                CLIError::EnvError(format!(
                    "Invalid DEPOSIT_PRIVATE_KEYS environment variable. Invalid private key: {}",
                    key
                ))
            })
        })
        .collect::<Result<Vec<H256>, CLIError>>()?;
    for &key in &keys {
        if key.is_zero() {
            return Err(CLIError::EnvError(format!(
                "Invalid DEPOSIT_PRIVATE_KEYS: Zero private key",
            ))
            .into());
        }
    }
    Ok(keys)
}

fn load_withdrawal_private_key() -> anyhow::Result<H256> {
    let withdrawal_private_key = env::var("WITHDRAWAL_PRIVATE_KEY").map_err(|_| {
        CLIError::EnvError("WITHDRAWAL_PRIVATE_KEY environment variable is not set".to_string())
    })?;
    let key = H256::from_str(&withdrawal_private_key).map_err(|_| {
        CLIError::EnvError(format!(
            "Invalid WITHDRAWAL_PRIVATE_KEY environment variable. Invalid private key: {}",
            withdrawal_private_key
        ))
    })?;
    if key.is_zero() {
        return Err(CLIError::EnvError(
            "Invalid WITHDRAWAL_PRIVATE_KEY: Zero private key".to_string(),
        )
        .into());
    }
    Ok(key)
}

pub fn load_max_gas_price() -> anyhow::Result<U256> {
    let max_gas_price_gwei = env::var("MAX_GAS_PRICE_IN_GWEI").unwrap_or("30".to_string());
    let max_gas_price_gwei: u32 = max_gas_price_gwei.parse().map_err(|_| {
        CLIError::EnvError(format!(
            "Invalid MAX_GAS_PRICE_IN_GWEI environment variable {}",
            max_gas_price_gwei
        ))
    })?;
    let max_gas_price = U256::from(max_gas_price_gwei) * U256::exp10(9);
    Ok(max_gas_price)
}

fn check_keys(keys: &Keys) -> anyhow::Result<()> {
    if keys.deposit_private_keys.is_empty() {
        return Err(CLIError::EnvError("No deposit private keys".to_string()).into());
    }
    if keys.deposit_addresses.contains(&keys.withdrawal_address) {
        return Err(
            CLIError::EnvError("Withdrawal address is also a deposit address".to_string()).into(),
        );
    }
    Ok(())
}

async fn check_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let client = ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)?;
    let chain_id = client.get_chainid().await?;
    let setting = Settings::load()?;
    if chain_id != setting.blockchain.chain_id.into() {
        return Err(anyhow::anyhow!(
            "RPC URL chain id {} does not match the expected chain id {}",
            chain_id,
            setting.blockchain.chain_id
        ));
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
