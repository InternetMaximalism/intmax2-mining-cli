use availability::check_avaliability;
use configure::recover_withdrawal_private_key;
use console::{initialize_console, print_status};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::{
        env_config::EnvConfig, env_validation::validate_env_config, errors::CLIError,
        network::get_network,
    },
};

pub mod accounts_status;
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
    let withdrawal_private_key = recover_withdrawal_private_key(&config)?;
    validate_env_config(&config).await?;
    config.export_to_env()?;

    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    let start_key_number =
        accounts_status::accounts_status(&mut state, config.mining_times, withdrawal_private_key)
            .await?;

    // wait for prover to be ready
    initialize_console();
    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    // main loop
    match mode {
        RunMode::Mining => {
            mining_loop(
                &mut state,
                withdrawal_private_key,
                start_key_number,
                config.mining_unit,
                config.mining_times,
            )
            .await?
        }
        RunMode::Claim => claim_loop(&mut state, withdrawal_private_key).await?,
        RunMode::Exit => exit_loop(&mut state, withdrawal_private_key).await?,
        _ => unreachable!(),
    }

    Ok(())
}
