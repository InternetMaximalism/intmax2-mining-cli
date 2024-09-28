use dialoguer::{Confirm, Input, Password, Select};
use ethers::types::{H256, U256};

use crate::{
    external_api::contracts::utils::get_address,
    utils::{
        config::Settings,
        encryption::{decrypt, encrypt},
        env_config::EnvConfig,
        env_validation::validate_rpc_url,
        network::get_network,
    },
};

pub async fn new_config() -> anyhow::Result<EnvConfig> {
    let rpc_url: String = input_rpc_url().await?;
    let default_env = Settings::load()?.env;
    let use_default = Confirm::new()
        .with_prompt(format!("Use default settings for max gas price ({} gwei), mining unit ({} ETH) and mining times ({})?", default_env.default_max_gas_price, default_env.default_mining_unit, default_env.default_mining_times))
        .default(true)
        .interact()?;
    let (max_gas_price, mining_unit, mining_times) = if use_default {
        (
            ethers::utils::parse_units(default_env.default_max_gas_price, "gwei")
                .unwrap()
                .into(),
            ethers::utils::parse_ether(default_env.default_mining_unit).unwrap(),
            default_env.default_mining_times,
        )
    } else {
        let max_gas_price = input_max_gas_price()?;
        let mining_unit = input_mining_unit()?;
        let mining_times = input_mining_times()?;
        (max_gas_price, mining_unit, mining_times)
    };
    let withdrawal_private_key: H256 = input_withdrawal_private_key()?;
    let (encrypt, keys, encrypted_keys) = input_encryption(withdrawal_private_key)?;
    let config = EnvConfig {
        rpc_url,
        max_gas_price,
        encrypt,
        withdrawal_private_key: keys,
        encrypted_withdrawal_private_key: encrypted_keys,
        mining_unit,
        mining_times,
    };
    Ok(config)
}

pub async fn modify_config(config: &EnvConfig) -> anyhow::Result<EnvConfig> {
    let key = recover_withdrawal_private_key(config)?;
    let modify_rpc = Confirm::new()
        .with_prompt(format!("Modify RPC URL {}?", config.rpc_url))
        .default(false)
        .interact()?;
    let rpc_url = if modify_rpc {
        input_rpc_url().await?
    } else {
        config.rpc_url.clone()
    };
    let modify_max_gas_price = Confirm::new()
        .with_prompt(format!(
            "Modify max gas price {} GWei?",
            ethers::utils::format_units(config.max_gas_price, "gwei").unwrap()
        ))
        .default(false)
        .interact()?;
    let max_gas_price = if modify_max_gas_price {
        input_max_gas_price()?
    } else {
        config.max_gas_price
    };

    let modify_withdrawal_address = Confirm::new()
        .with_prompt(format!("Modify withdrawal account {:?}?", get_address(key)))
        .default(false)
        .interact()?;
    let withdrawal_private_key = if modify_withdrawal_address {
        input_withdrawal_private_key()?
    } else {
        key
    };
    let modify_encryption = Confirm::new()
        .with_prompt(format!(
            "Modify encryption current={}?",
            config.encrypted_withdrawal_private_key.is_some()
        ))
        .default(false)
        .interact()?;
    let (encrypt, keys, encrypted_keys) = if modify_encryption {
        input_encryption(withdrawal_private_key)?
    } else {
        (
            config.encrypt,
            config.withdrawal_private_key.clone(),
            config.encrypted_withdrawal_private_key.clone(),
        )
    };
    let config = EnvConfig {
        rpc_url,
        max_gas_price,
        encrypt,
        withdrawal_private_key: keys,
        encrypted_withdrawal_private_key: encrypted_keys,
        mining_unit: config.mining_unit,
        mining_times: config.mining_times,
    };
    Ok(config)
}

async fn input_rpc_url() -> anyhow::Result<String> {
    loop {
        let items = ["Alchemy", "Infura", "Other"];
        let selection = Select::new()
            .with_prompt("Choose RPC provider")
            .items(&items)
            .default(0)
            .interact()?;
        let rpc_url = match selection {
            0 => input_alchemy_url().await?,
            1 => input_infura_url().await?,
            2 => input_custom_url().await?,
            _ => unreachable!(),
        };
        match validate_rpc_url(&rpc_url).await {
            Ok(_) => break Ok(rpc_url),
            Err(_) => {
                println!("Invalid RPC URL");
            }
        }
    }
}

async fn input_alchemy_url() -> anyhow::Result<String> {
    let alchemy_api_key: String = Input::new().with_prompt("Alchemy API Key").interact()?;
    let alchemy_url = format!(
        "https://eth-{}.g.alchemy.com/v2/{}",
        get_network(),
        alchemy_api_key
    );
    Ok(alchemy_url)
}

async fn input_infura_url() -> anyhow::Result<String> {
    let infura_project_id: String = Input::new().with_prompt("Infura Project ID").interact()?;
    let infura_url = format!(
        "https://{}.infura.io/v3/{}",
        get_network(),
        infura_project_id
    );
    Ok(infura_url)
}

async fn input_custom_url() -> anyhow::Result<String> {
    let rpc_url: String = Input::new()
        .with_prompt(format!("Custom RPC of {}", get_network()))
        .validate_with(|rpc_url: &String| {
            if rpc_url.starts_with("http") {
                Ok(())
            } else {
                Err("Invalid RPC URL")
            }
        })
        .interact()?;
    Ok(rpc_url)
}

fn input_max_gas_price() -> anyhow::Result<U256> {
    let max_gas_price: String = Input::new()
        .with_prompt("Max gas price for transactions in GWei")
        .default("30".to_string())
        .validate_with(|max_gas_price: &String| {
            let result = ethers::utils::parse_units(max_gas_price, "gwei");
            if let Ok(x) = result {
                if x > ethers::utils::parse_units("210", "gwei").unwrap() {
                    return Err("Gas price too high");
                }
                Ok(())
            } else {
                Err("Invalid gas price")
            }
        })
        .interact()?;
    let max_gas_price = ethers::utils::parse_units(max_gas_price, "gwei")
        .unwrap()
        .into();
    Ok(max_gas_price)
}

fn input_mining_unit() -> anyhow::Result<U256> {
    let items = vec!["0.1 ETH", "1.0 ETH"];
    let selection = Select::new()
        .with_prompt("Choose mining unit (amount per deposit)")
        .items(&items)
        .default(0)
        .interact()?;
    let mining_unit = match selection {
        0 => ethers::utils::parse_ether("0.1").unwrap(),
        1 => ethers::utils::parse_ether("1").unwrap(),
        _ => unreachable!(),
    };
    Ok(mining_unit)
}

fn input_mining_times() -> anyhow::Result<u64> {
    let items = vec!["10 times", "100 times"];
    let selection = Select::new()
        .with_prompt("Choose mining times (number of deposits)")
        .items(&items)
        .default(0)
        .interact()?;
    let mining_times = match selection {
        0 => 10,
        1 => 100,
        _ => unreachable!(),
    };
    Ok(mining_times)
}

fn input_withdrawal_private_key() -> anyhow::Result<H256> {
    let withdrawal_private_key: String = Password::new()
        .with_prompt(format!("Withdrawal private key of {}", get_network()))
        .validate_with(|input: &String| validate_private_key_with_duplication_check(&[], input))
        .interact()?;
    let withdrawal_private_key: H256 = withdrawal_private_key.parse().unwrap();
    let withdrawal_address = get_address(withdrawal_private_key);
    println!("Withdrawal Address: {:?}", withdrawal_address);
    Ok(withdrawal_private_key)
}

fn input_encryption(
    withdrawal_private_key: H256,
) -> anyhow::Result<(bool, Option<H256>, Option<Vec<u8>>)> {
    let do_encrypt = Confirm::new()
        .with_prompt("Do you set password to encrypt private keys?")
        .default(true)
        .interact()?;
    let key = if !do_encrypt {
        Some(withdrawal_private_key)
    } else {
        None
    };
    let encrypted_key = if do_encrypt {
        let password = Password::new()
            .with_prompt("Password to encrypt private key")
            .with_confirmation("Confirm password", "Passwords do not match")
            .interact()?;
        let encrypted_keys = encrypt(&password, &withdrawal_private_key)?;
        Some(encrypted_keys)
    } else {
        None
    };
    Ok((do_encrypt, key, encrypted_key))
}

fn validate_private_key_with_duplication_check(
    private_keys: &[H256],
    input: &str,
) -> Result<(), &'static str> {
    let result: Result<H256, _> = input.parse();
    match result {
        Ok(x) => {
            if x == H256::zero() {
                return Err("Invalid private key");
            }
            if private_keys.contains(&x) {
                return Err("Duplicated private key");
            }
            Ok(())
        }
        Err(_) => return Err("Invalid private key"),
    }
}

pub fn recover_withdrawal_private_key(config: &EnvConfig) -> anyhow::Result<H256> {
    let key = if !config.encrypt {
        config.withdrawal_private_key.clone().unwrap()
    } else {
        let password = Password::new().with_prompt("Password").interact()?;
        let key: H256 = decrypt(
            &password,
            config.encrypted_withdrawal_private_key.as_ref().unwrap(),
        )
        .map_err(|_| anyhow::anyhow!("Invalid password"))?;
        key
    };
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_config() {
        new_config().await.unwrap().save_to_file().unwrap();
    }

    #[tokio::test]
    async fn test_modify_config() {
        let config = EnvConfig::load_from_file().unwrap();
        modify_config(&config).await.unwrap();
    }

    #[tokio::test]
    async fn test_confirm() {
        Confirm::new().with_prompt("ADD").interact().unwrap();
    }
}
