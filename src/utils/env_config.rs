use std::path::PathBuf;

use ethers::types::U256;
use serde::{Deserialize, Serialize};

use crate::{state::keys::Keys, utils::network::get_network};

fn _env_config_path() -> PathBuf {
    PathBuf::from(format!("env.{}.json", get_network()))
}

// Structure for setting and getting env
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvConfig {
    pub rpc_url: String,
    pub max_gas_price: U256,
    pub keys: Option<Keys>,
    pub encrypted_keys: Option<Vec<u8>>,
    pub mining_unit: U256,
    pub mining_times: u64,
}

// string version of EnvConfig
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvConfigString {
    pub rpc_url: String,
    pub max_gas_price: String,
    pub keys: Option<String>,
    pub encrypted_keys: Option<String>,
    pub mining_unit: String,  // "0.1" or "1"
    pub mining_times: String, // "10" or "100"
}

impl EnvConfig {
    pub fn to_string(&self) -> EnvConfigString {
        let max_gas_price = ethers::utils::format_units(self.mining_unit, "gwei").unwrap();
        let keys = self
            .keys
            .clone()
            .map(|keys| serde_json::to_string(&keys).unwrap());
        let encrypted_keys = self
            .encrypted_keys
            .clone()
            .map(|encrypted_keys| hex::encode(encrypted_keys));
        let mining_unit = ethers::utils::format_units(self.mining_unit, "ether").unwrap();
        let mining_times = self.mining_times.to_string();
        EnvConfigString {
            rpc_url: self.rpc_url.clone(),
            max_gas_price,
            keys,
            encrypted_keys,
            mining_unit,
            mining_times,
        }
    }

    pub fn from_string(value: &EnvConfigString) -> anyhow::Result<Self> {
        let _max_gas_price: U256 = ethers::utils::parse_units(value.max_gas_price.clone(), "gwei")
            .map_err(|_| anyhow::anyhow!("failed to parse MAX_GAS_PRICE"))?
            .into();

        todo!()
    }
}

// impl EnvConfig {
//     pub fn load_from_file() -> anyhow::Result<Self> {
//         let file = std::fs::File::open(&env_config_path())?;
//         let reader = BufReader::new(file);
//         let config: EnvConfig = serde_json::from_reader(reader)?;
//         Ok(config)
//     }

//     pub fn save_to_file(&self) -> anyhow::Result<()> {
//         let input = serde_json::to_vec_pretty(self)?;
//         create_file_with_content(&env_config_path(), &input)?;
//         Ok(())
//     }

//     pub fn export_to_env(&self) -> anyhow::Result<()> {
//         // validation
//         if self.keys.is_some() && self.encrypted_keys.is_some() {
//             anyhow::bail!("Both keys and encrypted_keys are set in the configuration file. Please set only one of them.");
//         } else if self.keys.is_none() && self.encrypted_keys.is_none() {
//             anyhow::bail!("Neither keys nor encrypted_keys are set in the configuration file. Please set one of them.");
//         }
//         env::set_var("RPC_URL", &self.rpc_url);
//         env::set_var(
//             "MAX_GAS_PRICE",
//             &serde_json::to_string(&self.max_gas_price).unwrap(),
//         );
//         if let Some(keys) = &self.keys {
//             env::set_var("ENCRYPTION", "false");
//             keys.export_to_env();
//         } else if let Some(encrypted_keys) = &self.encrypted_keys {
//             env::set_var("ENCRYPTION", "true");
//             env::set_var("ENCRYPTED_KEYS", &serde_json::to_string(encrypted_keys)?);
//         }
//         env::set_var("MINING_UNIT", format!("{:?}", self.mining_unit));
//         env::set_var("MINING_TIMES", self.mining_times.to_string());
//         Ok(())
//     }

//     // import env config from env. Only checks format of env, not the correctness of the values
//     pub fn import_from_env() -> anyhow::Result<Self> {
//         let rpc_url = env::var("RPC_URL")
//             .map_err(|_| anyhow::Error::msg("RPC_URL environment variable is not set"))?;
//         let max_gas_price = env::var("MAX_GAS_PRICE").unwrap_or("30000000000".to_string());
//         let max_gas_price: U256 = serde_json::from_str(&max_gas_price).map_err(|_| {
//             anyhow::anyhow!(
//                 "Invalid MAX_GAS_PRICE environment variable {}",
//                 max_gas_price,
//             )
//         })?;
//         let encryption = env::var("ENCRYPTION").unwrap_or("false".to_string());
//         if encryption != "true" && encryption != "false" {
//             anyhow::bail!("ENCRYPTION environment variable must be either 'true' or 'false'");
//         }
//         let encryption = encryption == "true";
//         let encrypted_keys = if encryption {
//             let encrypted_keys = env::var("ENCRYPTED_KEYS").map_err(|_| {
//                 anyhow::Error::msg("ENCRYPTED_KEYS environment variable is not set")
//             })?;
//             let encrypted_keys: Vec<u8> = serde_json::from_str(&encrypted_keys)
//                 .map_err(|_| anyhow::Error::msg("Invalid ENCRYPTED_KEYS environment variable"))?;
//             Some(encrypted_keys)
//         } else {
//             None
//         };
//         let keys = if !encryption {
//             let keys = Keys::import_from_env()?;
//             Some(keys)
//         } else {
//             None
//         };
//         let mining_unit = env::var("MINING_UNIT").unwrap_or("100_000_000_000_000_000".to_string());
//         let mining_unit: U256 = mining_unit
//             .parse()
//             .map_err(|_| anyhow::Error::msg("Invalid MINING_UNIT environment variable"))?;
//         let mining_times = env::var("MINING_TIMES").unwrap_or("10".to_string());
//         let mining_times: u64 = mining_times
//             .parse()
//             .map_err(|_| anyhow::Error::msg("Invalid MINING_TIMES environment variable"))?;

//         Ok(EnvConfig {
//             rpc_url,
//             max_gas_price,
//             keys,
//             encrypted_keys,
//             mining_unit,
//             mining_times,
//         })
//     }
// }

// impl Keys {
//     pub fn export_to_env(&self) {
//         let deposit_private_keys = serde_json::to_string(&self.deposit_private_keys).unwrap();
//         env::set_var("DEPOSIT_PRIVATE_KEYS", &deposit_private_keys);
//         let withdrawal_private_key = format!("{:?}", self.withdrawal_private_key);
//         env::set_var("WITHDRAWAL_PRIVATE_KEY", withdrawal_private_key);
//     }

//     pub fn import_from_env() -> anyhow::Result<Self> {
//         let deposit_private_keys = env::var("DEPOSIT_PRIVATE_KEYS").map_err(|_| {
//             anyhow::Error::msg("DEPOSIT_PRIVATE_KEYS environment variable is not set")
//         })?;
//         let deposit_private_keys: Vec<H256> = serde_json::from_str(&deposit_private_keys).map_err(|_| {
//             anyhow::Error::msg(
//                 "Invalid DEPOSIT_PRIVATE_KEYS environment variable. Please set as format of DEPOSIT_PRIVATE_KEYS='[\"0xa...\", \"0xb...\"]'",
//             )
//         })?;
//         let withdrawal_private_key = env::var("WITHDRAWAL_PRIVATE_KEY").map_err(|_| {
//             anyhow::Error::msg("WITHDRAWAL_PRIVATE_KEY environment variable is not set")
//         })?;
//         let withdrawal_private_key: H256 = withdrawal_private_key.parse().map_err(|_| {
//             anyhow::Error::msg(
//                 "Invalid WITHDRAWAL_PRIVATE_KEY environment variable. Please set as format of WITHDRAWAL_PRIVATE_KEY='0xa...'",
//             )
//         })?;
//         Ok(Keys::new(deposit_private_keys, withdrawal_private_key))
//     }
// }

#[cfg(test)]
mod tests {

    // #[test]
    // fn load_env_test() {
    //     dotenv::dotenv().ok();
    //     let config = super::Keys::import_from_env().unwrap();
    // }

    // #[test]
    // fn test_keys_export_to_env() {
    //     let keys = super::Keys::new(
    //         vec![ethers::types::H256::random(), ethers::types::H256::random()],
    //         ethers::types::H256::random(),
    //     );
    //     keys.export_to_env();
    //     let keys_recovered = super::Keys::import_from_env().unwrap();
    //     assert_eq!(keys, keys_recovered);
    // }

    // #[test]
    // fn test_env_config_export_to_env() {
    //     let env_config = super::EnvConfig {
    //         rpc_url: "http://localhost:8545".to_string(),
    //         max_gas_price: 30_000_000_000u64.into(),
    //         keys: Some(super::Keys::new(
    //             vec![ethers::types::H256::random(), ethers::types::H256::random()],
    //             ethers::types::H256::random(),
    //         )),
    //         encrypted_keys: None,
    //         mining_unit: 100_000_000_000_000_000u128.into(),
    //         mining_times: 10,
    //     };
    //     env_config.export_to_env().unwrap();
    //     let env_config_recovered = super::EnvConfig::import_from_env().unwrap();
    //     assert_eq!(env_config, env_config_recovered);
    // }

    use ethers::{types::U256, utils::format_units};

    #[test]
    fn mini_test() {
        // 0.1 ETHに相当するwei値（100000000000000000 wei）
        let amount_in_wei = U256::from(100000000000000000u128);

        // weiからETHに変換
        let amount_in_eth = format_units(amount_in_wei, "ether").unwrap();

        // 結果を表示
        println!("{} wei = {} ETH", amount_in_wei, amount_in_eth);
    }
}
