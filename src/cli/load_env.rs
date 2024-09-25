use ethers::providers::Middleware;
use ethers::types::{Address, H256, U256};
use std::env;
use std::str::FromStr;

use crate::external_api::contracts::utils::get_address;
use crate::state::keys::{ClaimKeys, MiningKeys};
use crate::state::mode::RunMode;
use crate::utils::config::Settings;
use crate::utils::errors::CLIError;

#[derive(Debug, Clone, PartialEq)]
pub enum Config {
    Mining(MiningConfig),
    Claim(ClaimConfig),
    Exit(ExitConfig),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiningConfig {
    pub keys: MiningKeys,
    pub mining_unit: U256,
    pub mining_times: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimConfig {
    pub keys: ClaimKeys,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExitConfig {
    pub keys: MiningKeys,
}

pub async fn load_env(mode: RunMode) -> anyhow::Result<Config> {
    // test load global variables
    let _rpc_url = load_rpc_url().await?;
    let _max_gas_price = load_max_gas_price()?;

    let keys = match mode {
        RunMode::Mining => {
            let mining_unit = load_mining_unit()?;
            let mining_times = load_mining_times()?;

            let deposit_private_keys = load_deposit_private_keys()?;
            let withdrawal_address = load_withdrawal_address()?;
            let keys = MiningKeys::new(deposit_private_keys, withdrawal_address);
            check_mining_keys(&keys)?;
            Config::Mining(MiningConfig {
                keys,
                mining_unit,
                mining_times,
            })
        }
        RunMode::Claim => {
            let deposit_private_keys = load_deposit_private_keys()?;
            let claim_private_key = load_claim_private_key()?;
            let keys = ClaimKeys::new(deposit_private_keys, claim_private_key);
            check_claim_keys(&keys)?;
            Config::Claim(ClaimConfig { keys })
        }
        RunMode::Exit => {
            let deposit_private_keys = load_deposit_private_keys()?;
            let withdrawal_address = load_withdrawal_address()?;
            let keys = MiningKeys::new(deposit_private_keys, withdrawal_address);
            check_mining_keys(&keys)?;
            Config::Exit(ExitConfig { keys })
        }
    };
    Ok(keys)
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

fn load_claim_private_key() -> anyhow::Result<H256> {
    let claim_private_key = env::var("CLAIM_PRIVATE_KEY").map_err(|_| {
        CLIError::EnvError("CLAIM_PRIVATE_KEY environment variable is not set".to_string())
    })?;
    let key = H256::from_str(&claim_private_key).map_err(|_| {
        CLIError::EnvError(format!(
            "Invalid CLAIM_PRIVATE_KEY environment variable. Invalid private key: {}",
            claim_private_key
        ))
    })?;
    if key.is_zero() {
        return Err(
            CLIError::EnvError("Invalid CLAIM_PRIVATE_KEY: Zero private key".to_string()).into(),
        );
    }
    Ok(key)
}

fn load_withdrawal_address() -> anyhow::Result<Address> {
    let withdrawal_address = env::var("WITHDRAWAL_ADDRESS").map_err(|_| {
        CLIError::EnvError("WITHDRAWAL_ADDRESS environment variable is not set".to_string())
    })?;
    let address = Address::from_str(&withdrawal_address).map_err(|_| {
        CLIError::EnvError(format!(
            "Invalid WITHDRAWAL_ADDRESS: {}",
            withdrawal_address
        ))
    })?;
    if address == Address::default() {
        return Err(
            CLIError::EnvError(format!("Invalid WITHDRAWAL_ADDRESS: Zero address",)).into(),
        );
    }
    Ok(address)
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

fn check_mining_keys(keys: &MiningKeys) -> anyhow::Result<()> {
    if keys.deposit_private_keys.is_empty() {
        return Err(CLIError::EnvError("No deposit private keys".to_string()).into());
    }
    if keys.deposit_addresses.contains(&keys.withdrawal_address) {
        return Err(
            CLIError::EnvError("Withdrawal address is also a deposit address".to_string()).into(),
        );
    }

    // check claim address if it is set in the environment
    match load_claim_private_key() {
        Ok(claim_private_key) => {
            let claim_address = get_address(claim_private_key);
            if keys.deposit_addresses.contains(&claim_address) {
                return Err(CLIError::EnvError(
                    "Claim address is also a deposit address".to_string(),
                )
                .into());
            }
            if keys.withdrawal_address == claim_address {
                return Err(CLIError::EnvError(
                    "Claim address is also a withdrawal address".to_string(),
                )
                .into());
            }
        }
        Err(_) => {}
    }

    Ok(())
}

fn check_claim_keys(keys: &ClaimKeys) -> anyhow::Result<()> {
    if keys.deposit_private_keys.is_empty() {
        return Err(CLIError::EnvError("No deposit private keys".to_string()).into());
    }
    if keys.deposit_addresses.contains(&keys.claim_address) {
        return Err(
            CLIError::EnvError("Claim address is also a deposit address".to_string()).into(),
        );
    }

    match load_withdrawal_address() {
        Ok(withdrawal_address) => {
            if keys.deposit_addresses.contains(&withdrawal_address) {
                return Err(CLIError::EnvError(
                    "Withdrawal address is also a deposit address".to_string(),
                )
                .into());
            }
            if keys.claim_address == withdrawal_address {
                return Err(CLIError::EnvError(
                    "Claim address is also a withdrawal address".to_string(),
                )
                .into());
            }
        }
        Err(_) => {}
    }
    Ok(())
}

async fn check_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let client = ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)?;
    let chain_id = client.get_chainid().await?;
    let setting = Settings::new()?;
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
