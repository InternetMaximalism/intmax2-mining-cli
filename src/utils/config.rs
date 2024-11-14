use std::path::PathBuf;

use config::{Config, File};
use serde::Deserialize;

use crate::utils::network::get_network;

use super::{
    errors::CLIError,
    file::{create_file_with_content, get_data_path},
    network::Network,
};

fn config_path(network: Network) -> PathBuf {
    get_data_path()
        .unwrap()
        .join(format!("config.{}.toml", network))
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: Api,
    pub blockchain: Blockchain,
    pub service: Service,
    pub env: Env,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Api {
    pub availability_server_url: String,
    pub withdrawal_gnark_prover_url: String,
    pub claim_gnark_prover_url: String,
    pub circulation_server_url: String,
    pub gas_server_url: String,
    pub tree_data_repository: String,
    pub tree_data_directory: String,
    pub tree_data_branch: String,
    pub sync_tree_data_interval_in_sec: u64,
    pub gnark_get_proof_cooldown_in_sec: u64,
    pub withdrawal_server_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Blockchain {
    pub chain_id: u64,
    pub int1_address: String,
    pub minter_address: String,
    pub token_address: String,
    pub int1_deployed_block: u64,
    pub single_deposit_gas: u64,
    pub single_claim_gas: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Env {
    pub default_max_gas_price: String,
    pub default_mining_unit: String,
    pub default_mining_times: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    pub repository_url: String,
    pub mining_min_cooldown_in_sec: u64,
    pub mining_max_cooldown_in_sec: u64,
    pub loop_cooldown_in_sec: u64,
    pub high_gas_retry_inverval_in_sec: u64,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let s = Config::builder()
            .add_source(File::from(config_path(get_network())))
            .build()?;
        let s = s
            .try_deserialize()
            .map_err(|e| CLIError::ParseError(format!("Failed to parse config: {:?}", e)))?;
        Ok(s)
    }
}

const BASE_SEPOLIA_CONFIG: &'static [u8] = include_bytes!("../../config/config.base-sepolia.toml");
const BASE_CONFIG: &'static [u8] = include_bytes!("../../config/config.base.toml");
const MAINNET_CONFIG: &'static [u8] = include_bytes!("../../config/config.mainnet.toml");

pub fn create_config_files() -> anyhow::Result<()> {
    create_file_with_content(&config_path(Network::BaseSepolia), BASE_SEPOLIA_CONFIG)?;
    create_file_with_content(&config_path(Network::Base), BASE_CONFIG)?;
    create_file_with_content(&config_path(Network::Mainnet), MAINNET_CONFIG)?;
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_create_config_files() {
        super::create_config_files().unwrap();
    }
}
