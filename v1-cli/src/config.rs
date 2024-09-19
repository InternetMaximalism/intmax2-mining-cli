use std::{
    env,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

fn config_name() -> &'static str {
    let network = env::var("NETWORK").unwrap_or_else(|_| "testnet".into());
    match network.as_str() {
        "testnet" => "config.testnet",
        "localnet" => "config.localnet",
        _ => panic!("Unsupported network"),
    }
}

fn user_settings_path() -> &'static str {
    let network = env::var("NETWORK").unwrap_or_else(|_| "testnet".into());
    match network.as_str() {
        "testnet" => "data/user_settings.testnet.json",
        "localnet" => "data/user_settings.localnet.json",
        _ => panic!("Unsupported network"),
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: Api,
    pub blockchain: Blockchain,
    pub service: Service,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(config_name()))
            .build()?;
        s.try_deserialize()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Api {
    pub availability_server_url: String,
    pub withdrawal_gnark_prover_url: String,
    pub claim_gnark_prover_url: String,
    pub tree_data_repository: String,
    pub gnark_get_proof_cooldown_in_sec: u64,
    pub withdrawal_server_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Blockchain {
    pub chain_id: u64,
    pub int1_address: String,
    pub minter_address: String,
    pub single_deposit_gas_fee: String,
    pub sinlge_claim_gas_fee: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    pub mining_max_cooldown_in_sec: u64,
    pub claim_max_cooldown_in_sec: u64,
    pub main_loop_cooldown_in_sec: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MiningAmount {
    OneTenth,
    One,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InitialDeposit {
    One,
    Ten,
    Hundred,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub rpc_url: String,
    pub mining_amount: MiningAmount,
    pub initial_deposit: InitialDeposit,
    pub remaining_deposits: u64,
}

impl UserSettings {
    pub fn new() -> anyhow::Result<Self> {
        let file = std::fs::File::open(&user_settings_path())?;
        let reader = BufReader::new(file);
        let settings: UserSettings = serde_json::from_reader(reader)?;
        Ok(settings)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&user_settings_path())?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_config() {
        let settings = super::Settings::new().unwrap();
        dbg!(settings);
    }
}
