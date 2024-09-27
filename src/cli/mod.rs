use availability::check_avaliability;
use configure::recover_keys;
use console::{initialize_console, print_status};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::{
        env_config::EnvConfig, env_validation::validate_env_config, errors::CLIError,
        network::get_network,
    },
};

pub mod availability;
pub mod balance_validation;
pub mod configure;
pub mod console;
pub mod interactive;

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    println!(
        "Welcome to the INTMAX mining CLI!. Network: {}, Mode: {:?}",
        get_network(),
        mode
    );
    check_avaliability().await?;

    let mode = if mode == RunMode::Interactive {
        interactive::interactive().await?
    } else {
        mode.clone()
    };

    // export env config
    match EnvConfig::load_from_file() {
        Ok(config) => {
            config.export_to_env()?;
        }
        Err(_) => {}
    }

    let config = EnvConfig::import_from_env().map_err(|e| CLIError::EnvError(e.to_string()))?;
    let keys = recover_keys(&config)?;
    validate_env_config(&config, &keys).await?;
    config.export_to_env()?;

    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    // valance validation
    balance_validation::balance_validation(&mut state, mode, &config, &keys).await?;

    // wait for prover to be ready
    initialize_console();
    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    // main loop
    match mode {
        RunMode::Mining => {
            mining_loop(&mut state, &keys, config.mining_unit, config.mining_times).await?
        }
        RunMode::Claim => claim_loop(&mut state, &keys).await?,
        RunMode::Exit => exit_loop(&mut state, &keys).await?,
        _ => unreachable!(),
    }

    Ok(())
}
