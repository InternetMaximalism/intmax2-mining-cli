use std::{collections::HashMap, env};

use console::{style, Term};
use dialoguer::Select;
use ethers::types::Address;
use strum::IntoEnumIterator;

use crate::{
    cli::configure::select_network,
    state::mode::RunMode,
    utils::{env_config::EnvConfig, network::Network},
};

use super::configure::{modify_config, new_config};

pub async fn interactive() -> anyhow::Result<()> {
    let network = select_network()?;
    env::set_var("NETWORK", network.to_string());

    let existing_indices = EnvConfig::get_existing_indices(network);
    if existing_indices.is_empty() {
        println!("No existing indices found. Please create a new config.");
        let config = new_config(network).await?;
        config.save_to_file(0)?;
        config.export_to_env()?;
        return Ok(());
    }

    // list existing config files
    let mut items = existing_indices
        .iter()
        .map(|index| format!("#{}", index,))
        .collect::<Vec<String>>();
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
    config.save_to_file(config_number)?;
    config.export_to_env()?;

    // todo
    address_duplication_check()?;
    Ok(())
}

pub fn select_mode() -> anyhow::Result<RunMode> {
    let items = [
        format!(
            "{} {}",
            style("Mining:").bold(),
            style("performs mining by repeatedly executing deposits and withdrawals").dim()
        ),
        format!(
            "{} {}",
            style("Claim:").bold(),
            style("claims available ITX tokens").dim()
        ),
        format!(
            "{} {}",
            style("Exit:").bold(),
            style("withdraws all balances currently and cancels pending deposits").dim()
        ),
        format!(
            "{} {}",
            style("Export:").bold(),
            style("export deposit private keys").dim()
        ),
        format!(
            "{} {}",
            style("Check Update:").bold(),
            style("check for updates of this CLI").dim()
        ),
    ];
    let term = Term::stdout();
    term.clear_screen()?;
    let mode = Select::new()
        .with_prompt("Select mode (press ctrl+c to abort)")
        .items(&items)
        .default(0)
        .interact()?;
    let mode = match mode {
        0 => RunMode::Mining,
        1 => RunMode::Claim,
        2 => RunMode::Exit,
        3 => RunMode::Export,
        4 => RunMode::CheckUpdate,
        _ => unreachable!(),
    };
    Ok(mode)
}

fn address_duplication_check() -> anyhow::Result<()> {
    let mut address_to_network = HashMap::<Address, (Network, usize)>::new();
    for network in Network::iter() {
        for config_index in EnvConfig::get_existing_indices(network) {
            let config = EnvConfig::load_from_file(network, config_index)?;
            if address_to_network.get(&config.withdrawal_address).is_some() {
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
            } else {
                address_to_network.insert(config.withdrawal_address, (network, config_index));
            }
        }
    }

    Ok(())
}
