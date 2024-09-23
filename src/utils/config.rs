use std::{io::BufReader, path::PathBuf};

use anyhow::Context;
use config::{Config, File};
use serde::{Deserialize, Serialize};

use crate::utils::{file::create_file_with_content, network::get_network};

fn config_name() -> String {
    format!("config.{}", get_network())
}

fn user_settings_path() -> PathBuf {
    PathBuf::from(format!("data/user_settings.{}.json", get_network()))
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub api: Api,
    pub blockchain: Blockchain,
    pub service: Service,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name(&config_name()))
            .build()
            .context("Failed to load config")?;
        s.try_deserialize().context("Failed to deserialize config")
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
    pub token_address: String,
    pub single_deposit_gas_fee: String,
    pub single_claim_gas_fee: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    pub mining_max_cooldown_in_sec: u64,
    pub claim_max_cooldown_in_sec: u64,
    pub main_loop_cooldown_in_sec: u64,
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum MiningAmount {
//     OneTenth,
//     One,
// }

// #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub enum InitialDeposit {
//     One,
//     Ten,
//     Hundred,
// }

// #[derive(Clone, Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct UserSettings {
//     pub rpc_url: String,
//     pub mining_amount: MiningAmount,
//     pub initial_deposit: InitialDeposit,
//     pub max_deposits: usize,
// }

// impl UserSettings {
//     pub fn new() -> anyhow::Result<Self> {
//         let file =
//             std::fs::File::open(&user_settings_path()).context("Failed to open user settings")?;
//         let reader = BufReader::new(file);
//         let settings: UserSettings = serde_json::from_reader(reader)?;
//         Ok(settings)
//     }

//     pub fn save(&self) -> anyhow::Result<()> {
//         let input = serde_json::to_vec_pretty(self)?;
//         create_file_with_content(&user_settings_path(), &input)?;
//         Ok(())
//     }
// }
