use std::collections::HashMap;

use alloy::primitives::Address;
use console::style;
use dialoguer::Select;
use strum::IntoEnumIterator;

use crate::utils::{
    env_config::EnvConfig,
    network::{get_network, is_legacy, Network},
};

use super::configure::{modify_config, new_config};

pub async fn interactive() -> anyhow::Result<()> {
    if is_legacy() {
        legacy_select_or_create_config().await?;
    } else {
        select_or_create_config().await?;
    }
    address_duplication_check()?;
    Ok(())
}

fn address_duplication_check() -> anyhow::Result<()> {
    let mut address_to_network = HashMap::<Address, (Network, usize)>::new();
    for network in Network::iter() {
        for config_index in EnvConfig::get_existing_indices(network) {
            let config = EnvConfig::load_from_file(network, config_index)?;
            if let std::collections::hash_map::Entry::Vacant(e) = address_to_network.entry(config.withdrawal_address) {
                e.insert((network, config_index));
            } else {
                let (duplicated_network, duplicated_index) =
                    address_to_network.get(&config.withdrawal_address).unwrap();
                anyhow::bail!(
                    "Withdrawal address {} on {} config #{} is duplicated as {} config #{}. Please use a different address.",
                    config.withdrawal_address,
                    network,
                    config_index,
                    duplicated_network,
                    duplicated_index,
                );
            }
        }
    }
    Ok(())
}

async fn select_or_create_config() -> anyhow::Result<()> {
    let network = get_network();
    let existing_indices = EnvConfig::get_existing_indices(network);
    if existing_indices.is_empty() {
        println!("No existing indices found. Please create a new config.");
        let config = new_config(network).await?;
        config.save_to_file(0)?;
        config.export_to_env()?;
        return Ok(());
    }

    let mut items = Vec::new();
    for i in &existing_indices {
        let config = EnvConfig::load_from_file(network, *i)?;
        items.push(format!(
            "Config #{} (withdrawal address: {})",
            i, config.withdrawal_address
        ));
    }
    items.push("Create New Config".to_string());
    let selection = Select::new()
        .with_prompt("Please select config to use")
        .items(&items)
        .default(0)
        .interact()?;
    if selection == existing_indices.len() {
        let config = new_config(network).await?;
        let new_index = existing_indices.iter().max().unwrap() + 1;
        config.save_to_file(new_index)?;
        config.export_to_env()?;
        return Ok(());
    }
    let config_number = existing_indices[selection];
    let config = load_config_with_option(network, config_number).await?;
    config.save_to_file(config_number)?;
    config.export_to_env()?;
    Ok(())
}

async fn legacy_select_or_create_config() -> anyhow::Result<()> {
    let network = get_network();

    if !EnvConfig::is_file_exist(network, 0) {
        let config = new_config(network).await?;
        config.save_to_file(0)?;
        config.export_to_env()?;
        return Ok(());
    }
    let config = load_config_with_option(network, 0).await?;
    config.save_to_file(0)?;
    config.export_to_env()?;
    Ok(())
}

async fn load_config_with_option(
    network: Network,
    config_number: usize,
) -> anyhow::Result<EnvConfig> {
    let items = vec![
        format!(
            "{} {}",
            style("Continue:").bold(),
            style("continue with the existing config").dim()
        ),
        format!(
            "{} {}",
            style("Overwrite:").bold(),
            style("overwrite the existing config").dim()
        ),
        format!(
            "{} {}",
            style("Modify:").bold(),
            style("modify the existing config").dim()
        ),
    ];
    let selection = Select::new()
        .with_prompt(format!(
            "What do you want to do with for the config #{}?",
            config_number
        ))
        .items(&items)
        .default(0)
        .interact()?;
    let config = match selection {
        0 => EnvConfig::load_from_file(network, config_number)?,
        1 => new_config(network).await?,
        2 => {
            let config = EnvConfig::load_from_file(network, config_number)?;
            modify_config(&config).await?
        }
        _ => unreachable!(),
    };
    Ok(config)
}
