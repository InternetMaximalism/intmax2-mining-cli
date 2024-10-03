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

    let is_file_exists = EnvConfig::load_from_file(network).is_ok();
    if is_file_exists {
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
            .with_prompt("Config file already exists. What do you want to do?")
            .items(&items)
            .default(0)
            .interact()?;
        let config = match selection {
            0 => EnvConfig::load_from_file(network)?,
            1 => new_config(network).await?,
            2 => {
                let config = EnvConfig::load_from_file(network)?;
                modify_config(&config).await?
            }
            _ => unreachable!(),
        };
        config.save_to_file()?;
        config.export_to_env()?;
    } else {
        println!("Config file not found. Creating a new one.");
        let config = new_config(network).await?;
        config.save_to_file()?;
        config.export_to_env()?;
    };
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
    let mut address_to_network = HashMap::<Address, Network>::new();
    for network in Network::iter() {
        let config = EnvConfig::load_from_file(network).ok();
        if let Some(config) = config {
            if address_to_network.get(&config.withdrawal_address).is_some() {
                anyhow::bail!(
                    "Withdrawal address {} is duplicated on {} and {}. Please use a different address.",
                    config.withdrawal_address,
                    address_to_network.get(&config.withdrawal_address).unwrap(),
                    network
                );
            } else {
                address_to_network.insert(config.withdrawal_address, network);
            }
        }
    }

    Ok(())
}
