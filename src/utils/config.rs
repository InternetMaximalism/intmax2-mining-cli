use anyhow::Context;
use config::{Config, File};
use serde::Deserialize;

use crate::utils::network::get_network;

use super::errors::CLIError;

fn config_name() -> String {
    format!("config.{}", get_network())
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: Api,
    pub blockchain: Blockchain,
    pub service: Service,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Api {
    pub availability_server_url: String,
    pub withdrawal_gnark_prover_url: String,
    pub claim_gnark_prover_url: String,
    pub circulation_server_url: String,
    pub tree_data_repository: String,
    pub tree_data_directory: String,
    pub tree_data_branch: String,
    pub gnark_get_proof_cooldown_in_sec: u64,
    pub withdrawal_server_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Blockchain {
    pub chain_id: u64,
    pub int1_address: String,
    pub minter_address: String,
    pub token_address: String,
    pub single_deposit_gas: u64,
    pub single_claim_gas: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    pub repository_url: String,
    pub mining_max_cooldown_in_sec: u64,
    pub loop_cooldown_in_sec: u64,
    pub high_gas_retry_inverval_in_sec: u64,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name(&config_name()))
            .build()
            .context("Failed to load config")?;
        let s = s.try_deserialize().map_err(|e| {
            CLIError::ParseError(format!("Failed to parse {}.toml. {:?}", config_name(), e))
        })?;
        Ok(s)
    }
}
