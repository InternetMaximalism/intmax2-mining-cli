use alloy::primitives::{
    utils::{format_units, parse_units},
    Address, B256, U256,
};
use serde::{Deserialize, Serialize};
use std::{env, io::BufReader, path::PathBuf, str::FromStr as _};

use super::{
    config::Settings,
    file::{create_file_with_content, get_data_path},
    network::Network,
};

fn env_config_path(network: Network, i: usize) -> PathBuf {
    let i_str = if i == 0 {
        "".to_string() // no suffix for the first env for the legacy support
    } else {
        format!(".{}", i)
    };
    get_data_path()
        .unwrap()
        .join(format!("env.{}{}.json", network, i_str))
}

// Structure for setting and getting env
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvConfig {
    pub network: Network,
    pub rpc_url: String,
    pub max_gas_price: U256,
    pub encrypt: bool,
    pub withdrawal_address: Address,
    pub withdrawal_private_key: Option<B256>,
    pub encrypted_withdrawal_private_key: Option<Vec<u8>>,
    pub mining_unit: U256,
    pub mining_times: u64,
}

impl EnvConfig {
    pub fn get_existing_indices(network: Network) -> Vec<usize> {
        let mut i = 0;
        let mut indices = vec![];
        while Self::is_file_exist(network, i) {
            indices.push(i);
            i += 1;
        }
        indices
    }

    pub fn is_file_exist(network: Network, i: usize) -> bool {
        env_config_path(network, i).exists()
    }

    pub fn load_from_file(network: Network, i: usize) -> anyhow::Result<Self> {
        let file = std::fs::File::open(env_config_path(network, i)).map_err(|_| {
            anyhow::anyhow!(
                "Failed to open the config file at {:?}",
                env_config_path(network, i)
            )
        })?;
        let reader = BufReader::new(file);
        let config: EnvConfig = serde_json::from_reader(reader).map_err(|_| {
            anyhow::anyhow!("Config at {:?} is broken", env_config_path(network, i))
        })?;
        Ok(config)
    }

    pub fn save_to_file(&self, i: usize) -> anyhow::Result<()> {
        let input = serde_json::to_vec_pretty(self)?;
        create_file_with_content(&env_config_path(self.network, i), &input)?;
        Ok(())
    }

    pub fn export_to_env(&self) -> anyhow::Result<()> {
        let config_string = self.to_string()?;
        env::set_var("NETWORK", &config_string.network);
        env::set_var("RPC_URL", &config_string.rpc_url);
        env::set_var("MAX_GAS_PRICE", &config_string.max_gas_price);
        env::set_var("WITHDRAWAL_ADDRESS", &config_string.withdrawal_address);
        env::set_var("ENCRYPT", &config_string.encrypt);
        if self.encrypt {
            env::set_var(
                "ENCRYPTED_WITHDRAWAL_PRIVATE_KEY",
                config_string.encrypted_withdrawal_private_key.unwrap(),
            );
        } else {
            env::set_var(
                "WITHDRAWAL_PRIVATE_KEY",
                config_string.withdrawal_private_key.unwrap(),
            );
        }
        env::set_var("MINING_UNIT", &config_string.mining_unit);
        env::set_var("MINING_TIMES", &config_string.mining_times);
        Ok(())
    }

    // import env config from env. Only checks format of env, not the correctness of the values
    pub fn import_from_env() -> anyhow::Result<Self> {
        let network = env::var("NETWORK").unwrap_or(Network::default().to_string());
        let default_env = Settings::load()?.env;
        let rpc_url = env::var("RPC_URL")
            .map_err(|_| anyhow::Error::msg("RPC_URL environment variable is not set"))?;
        let max_gas_price = env::var("MAX_GAS_PRICE").unwrap_or(default_env.default_max_gas_price);
        let withdrawal_address = env::var("WITHDRAWAL_ADDRESS").map_err(|_| {
            anyhow::Error::msg("WITHDRAWAL_ADDRESS environment variable is not set")
        })?;
        let encrypt = env::var("ENCRYPT").unwrap_or("true".to_string());
        let withdrawal_private_key = env::var("WITHDRAWAL_PRIVATE_KEY").ok();
        let encrypted_withdrawal_private_key = env::var("ENCRYPTED_WITHDRAWAL_PRIVATE_KEY").ok();
        let mining_unit = env::var("MINING_UNIT").unwrap_or(default_env.default_mining_unit);
        let mining_times =
            env::var("MINING_TIMES").unwrap_or(default_env.default_mining_times.to_string());
        let config_string = EnvConfigString {
            network,
            rpc_url,
            max_gas_price,
            encrypt,
            withdrawal_address,
            withdrawal_private_key,
            encrypted_withdrawal_private_key,
            mining_unit,
            mining_times,
        };
        let config = EnvConfig::from_string(&config_string)?;
        Ok(config)
    }

    fn to_string(&self) -> anyhow::Result<EnvConfigString> {
        let network = format!("{}", self.network);
        let max_gas_price = format_units(self.max_gas_price, "gwei").unwrap();
        let encrypt = if self.withdrawal_private_key.is_some() {
            "false".to_string()
        } else if self.encrypted_withdrawal_private_key.is_some() {
            "true".to_string()
        } else {
            anyhow::bail!("Both keys and encrypted_keys are not set in the configuration file. Please set one of them.");
        };
        let withdrawal_address = format!("{:?}", self.withdrawal_address);
        let withdrawal_private_key = self.withdrawal_private_key.map(|key| format!("{:?}", key));
        let encrypted_withdrawal_private_key = self
            .encrypted_withdrawal_private_key
            .clone()
            .map(hex::encode);
        let mining_unit = format_units(self.mining_unit, "ether").unwrap();
        let mining_times = self.mining_times.to_string();
        Ok(EnvConfigString {
            network,
            rpc_url: self.rpc_url.clone(),
            max_gas_price,
            encrypt,
            withdrawal_address,
            withdrawal_private_key,
            encrypted_withdrawal_private_key,
            mining_unit,
            mining_times,
        })
    }

    fn from_string(value: &EnvConfigString) -> anyhow::Result<Self> {
        let network =
            Network::from_str(&value.network).map_err(|_| anyhow::anyhow!("Invalid network"))?;
        let max_gas_price: U256 = parse_units(&value.max_gas_price, "gwei")
            .map_err(|_| anyhow::anyhow!("failed to parse MAX_GAS_PRICE"))?
            .into();
        let encrypt = if value.encrypt == "true" {
            true
        } else if value.encrypt == "false" {
            false
        } else {
            anyhow::bail!("ENCRYPT must be either 'true' or 'false'");
        };

        if !encrypt && value.withdrawal_private_key.is_none() {
            anyhow::bail!("WITHDRAWAL_PRIVATE_KEY is not set.");
        } else if encrypt && value.encrypted_withdrawal_private_key.is_none() {
            anyhow::bail!("ENCRYPTED_WITHDRAWAL_PRIVATE_KEY is not set.");
        }
        let withdrawal_address: Address = value
            .withdrawal_address
            .parse()
            .map_err(|_| anyhow::anyhow!("failed to parse WITHDRAWAL_ADDRESS"))?;

        let withdrawal_private_key = if !encrypt {
            let withdrawal_private_key: B256 = value
                .withdrawal_private_key
                .as_ref()
                .unwrap()
                .parse()
                .map_err(|_| anyhow::anyhow!("failed to parse WITHDRAWAL_PRIVATE_KEY"))?;
            Some(withdrawal_private_key)
        } else {
            None
        };
        let encrypted_withdrawal_private_key = if encrypt {
            let encrypted_withdrawal_private_key: Vec<u8> = hex::decode(
                value.encrypted_withdrawal_private_key.as_ref().unwrap(),
            )
            .map_err(|_| anyhow::anyhow!("failed to parse ENCRYPTED_WITHDRAWAL_PRIVATE_KEY"))?;
            Some(encrypted_withdrawal_private_key)
        } else {
            None
        };

        let mining_unit: U256 = parse_units(&value.mining_unit, "ether")
            .map_err(|_| anyhow::anyhow!("failed to parse MINING_UNIT"))?
            .into();
        let mining_times: u64 = value
            .mining_times
            .parse()
            .map_err(|_| anyhow::anyhow!("failed to parse MINING_TIMES"))?;

        Ok(EnvConfig {
            network,
            rpc_url: value.rpc_url.clone(),
            max_gas_price,
            encrypt,
            withdrawal_address,
            withdrawal_private_key,
            encrypted_withdrawal_private_key,
            mining_unit,
            mining_times,
        })
    }
}

// string version of EnvConfig
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct EnvConfigString {
    network: String,
    rpc_url: String,
    max_gas_price: String,
    encrypt: String,
    withdrawal_address: String,
    withdrawal_private_key: Option<String>,
    encrypted_withdrawal_private_key: Option<String>,
    mining_unit: String,
    mining_times: String,
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{
        utils::{format_units, parse_units},
        B256, U256,
    };

    use crate::{
        external_api::contracts::utils::get_address_from_private_key, utils::network::Network,
    };

    #[test]
    fn load_env_test() {
        dotenv::dotenv().ok();
        let config = super::EnvConfig::import_from_env().unwrap();
        dbg!(config);
    }

    #[test]
    fn test_env_config_string_conversion() {
        let key = B256::random();
        let address = get_address_from_private_key(key);
        let env_config = super::EnvConfig {
            network: Network::Localnet,
            rpc_url: "http://localhost:8545".to_string(),
            max_gas_price: U256::from(30_000_000_000u64),
            encrypt: false,
            withdrawal_address: address,
            withdrawal_private_key: Some(key),
            encrypted_withdrawal_private_key: None,
            mining_unit: U256::from(100_000_000_000_000_000u128),
            mining_times: 10,
        };
        let env_config_string = env_config.to_string().unwrap();
        let env_config_recovered = super::EnvConfig::from_string(&env_config_string).unwrap();
        assert_eq!(env_config, env_config_recovered);
    }

    #[test]
    fn test_export_and_import_config() {
        let key = B256::random();
        let address = get_address_from_private_key(key);
        let env_config = super::EnvConfig {
            network: Network::Localnet,
            rpc_url: "http://localhost:8545".to_string(),
            max_gas_price: U256::from(30_000_000_000u64),
            withdrawal_address: address,
            encrypt: false,
            withdrawal_private_key: Some(key),
            encrypted_withdrawal_private_key: None,
            mining_unit: U256::from(100_000_000_000_000_000u128),
            mining_times: 10,
        };
        env_config.export_to_env().unwrap();

        let env_config_recovered = super::EnvConfig::import_from_env().unwrap();
        assert_eq!(env_config, env_config_recovered);
    }

    #[test]
    fn mini_test() {
        let amount = U256::from(100000000000000000u128);
        let amount_str: String = format_units(amount, "gwei").unwrap();
        let recover: U256 = parse_units(&amount_str, "gwei").unwrap().into();
        assert_eq!(amount, recover);
    }
}
