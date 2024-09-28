use dialoguer::{Confirm, Input, Password, Select};
use ethers::types::{H256, U256};

use crate::{
    external_api::contracts::utils::get_address,
    state::key::Keys,
    utils::{
        encryption::{decrypt, encrypt},
        env_config::EnvConfig,
        env_validation::validate_rpc_url,
        network::get_network,
    },
};

pub async fn new_config() -> anyhow::Result<EnvConfig> {
    let rpc_url: String = input_rpc_url().await?;
    let max_gas_price: U256 = input_max_gas_price()?;
    let mining_unit = input_mining_unit()?;
    let mining_times = input_mining_times()?;
    let deposit_private_key: H256 = {
        let deposit_private_key: String = Password::new()
            .with_prompt(format!("Deposit private key of {}", get_network()))
            .validate_with(|input: &String| validate_private_key_with_duplication_check(&[], input))
            .interact()?;
        let deposit_private_key: H256 = deposit_private_key.parse().unwrap();
        let deposit_address = get_address(deposit_private_key);
        println!("Deposit Address: {:?}", deposit_address);
        deposit_private_key
    };
    let is_more = Confirm::new()
        .with_prompt("Add more deposit private keys?")
        .default(false)
        .interact()?;
    let deposit_private_keys = if is_more {
        append_deposit_private_keys(&[deposit_private_key])?
    } else {
        vec![deposit_private_key]
    };
    let withdrawal_private_key: H256 = input_withdrawal_private_key(&deposit_private_keys)?;
    let (encrypt, keys, encrypted_keys) =
        input_encryption(&deposit_private_keys, withdrawal_private_key)?;
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
    let keys = recover_keys(config)?;

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

    println!("Deposit Addresses:");
    for deposit_address in keys.deposit_addresses.iter() {
        println!("{:?}", deposit_address);
    }
    let append_deposit_address = Confirm::new()
        .with_prompt("Append deposit accounts?")
        .default(false)
        .interact()?;
    let deposit_private_keys = if append_deposit_address {
        append_deposit_private_keys(&keys.deposit_private_keys)?
    } else {
        keys.deposit_private_keys.clone()
    };

    let modify_withdrawal_address = Confirm::new()
        .with_prompt(format!(
            "Modify withdrawal account {:?}?",
            keys.withdrawal_address
        ))
        .default(false)
        .interact()?;
    let withdrawal_private_key = if modify_withdrawal_address {
        input_withdrawal_private_key(&deposit_private_keys)?
    } else {
        keys.withdrawal_private_key
    };

    let modify_encryption = Confirm::new()
        .with_prompt(format!(
            "Modify encryption current={}?",
            config.encrypted_withdrawal_private_key.is_some()
        ))
        .default(false)
        .interact()?;
    let (encrypt, keys, encrypted_keys) = if modify_encryption {
        input_encryption(&deposit_private_keys, withdrawal_private_key)?
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
        let rpc_url: String = Input::new()
            .with_prompt(format!(
                "RPC URL of {}. We highly recommend using Alchemy's RPC",
                get_network()
            ))
            .validate_with(|rpc_url: &String| {
                if rpc_url.starts_with("http") {
                    Ok(())
                } else {
                    Err("Invalid RPC URL")
                }
            })
            .interact()?;
        match validate_rpc_url(&rpc_url).await {
            Ok(_) => break Ok(rpc_url),
            Err(_) => {
                println!("Wrong RPC URL")
            }
        }
    }
}

fn input_max_gas_price() -> anyhow::Result<U256> {
    let max_gas_price: String = Input::new()
        .with_prompt("Max gas price for transactions in GWei")
        .default("30".to_string())
        .validate_with(|max_gas_price: &String| {
            let result = ethers::utils::parse_units(max_gas_price, "gwei");
            if result.is_ok() {
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

fn append_deposit_private_keys(current_deposit_private_keys: &[H256]) -> anyhow::Result<Vec<H256>> {
    let mut deposit_private_keys = current_deposit_private_keys.to_vec();
    loop {
        let deposit_private_key: H256 = {
            let deposit_private_key: String = Password::new()
                .with_prompt(format!("Deposit private key of {}", get_network()))
                .validate_with(|input: &String| {
                    validate_private_key_with_duplication_check(&deposit_private_keys, input)
                })
                .interact()?;
            let deposit_private_key: H256 = deposit_private_key.parse().unwrap();
            let deposit_address = get_address(deposit_private_key);
            println!("Deposit Address: {:?}", deposit_address);
            deposit_private_key
        };
        deposit_private_keys.push(deposit_private_key);
        let is_more = Confirm::new()
            .with_prompt("Add more deposit private keys?")
            .default(false)
            .interact()?;
        if is_more {
            continue;
        } else {
            break;
        }
    }
    Ok(deposit_private_keys)
}

fn input_withdrawal_private_key(deposit_private_keys: &[H256]) -> anyhow::Result<H256> {
    let withdrawal_private_key: String = Password::new()
        .with_prompt(format!("Withdrawal private key of {}", get_network()))
        .validate_with(|input: &String| {
            validate_private_key_with_duplication_check(deposit_private_keys, input)
        })
        .interact()?;
    let withdrawal_private_key: H256 = withdrawal_private_key.parse().unwrap();
    let withdrawal_address = get_address(withdrawal_private_key);
    println!("Withdrawal Address: {:?}", withdrawal_address);
    Ok(withdrawal_private_key)
}

fn input_encryption(
    deposit_private_keys: &[H256],
    withdrawal_private_key: H256,
) -> anyhow::Result<(bool, Option<Keys>, Option<Vec<u8>>)> {
    let do_encrypt = Confirm::new()
        .with_prompt("Do you set password to encrypt private keys?")
        .default(true)
        .interact()?;
    let raw_keys = Keys::new(deposit_private_keys.to_vec(), withdrawal_private_key);
    let keys = if !do_encrypt {
        Some(raw_keys.clone())
    } else {
        None
    };
    let encrypted_keys = if do_encrypt {
        let password = Password::new()
            .with_prompt("Password to encrypt private key")
            .with_confirmation("Confirm password", "Passwords do not match")
            .interact()?;
        let encrypted_keys = encrypt(&password, &raw_keys)?;
        Some(encrypted_keys)
    } else {
        None
    };
    Ok((do_encrypt, keys, encrypted_keys))
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

pub fn recover_keys(config: &EnvConfig) -> anyhow::Result<Keys> {
    let keys = if !config.encrypt {
        config.withdrawal_private_key.clone().unwrap()
    } else {
        let password = Password::new().with_prompt("Password").interact()?;
        let keys: Keys = decrypt(
            &password,
            config.encrypted_withdrawal_private_key.as_ref().unwrap(),
        )
        .map_err(|_| anyhow::anyhow!("Invalid password"))?;
        keys
    };
    Ok(keys)
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
