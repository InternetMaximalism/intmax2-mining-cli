use dialoguer::Select;

use crate::{state::mode::RunMode, utils::env_config::EnvConfig};

use super::configure::{modify_config, new_config};

pub async fn interactive() -> anyhow::Result<RunMode> {
    let is_file_exists = EnvConfig::load_from_file().is_ok();

    if is_file_exists {
        let items = vec!["Continue", "Overwrite", "Modify"];
        let selection = Select::new()
            .with_prompt("Config file already exists. What do you want to do?")
            .items(&items)
            .default(0)
            .interact()?;
        let config = match selection {
            0 => EnvConfig::load_from_file()?,
            1 => new_config().await?,
            2 => {
                let config = EnvConfig::load_from_file()?;
                modify_config(&config).await?
            }
            _ => unreachable!(),
        };
        config.save_to_file()?;
        config.export_to_env()?;
    } else {
        println!("Config file not found. Creating a new one.");
        let config = new_config().await?;
        config.save_to_file()?;
        config.export_to_env()?;
    };

    let mode = Select::new()
        .with_prompt("Select mode")
        .items(&["Mining", "Claim", "Exit"])
        .default(0)
        .interact()?;
    let mode = match mode {
        0 => RunMode::Mining,
        1 => RunMode::Claim,
        2 => RunMode::Exit,
        _ => unreachable!(),
    };
    Ok(mode)
}
