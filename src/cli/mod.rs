use availability::check_avaliability;
use configure::recover_keys;
use console::print_status;

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

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    println!(
        "Welcome to the INTMAX mining CLI!. Network: {}, Mode: {:?}",
        get_network(),
        mode
    );
    check_avaliability().await?;

    if mode == RunMode::Config {
        configure::configure().await?;
        return Ok(());
    }

    // load env config
    let config = EnvConfig::import_from_env().map_err(|e| CLIError::EnvError(e.to_string()))?;
    let keys = recover_keys(&config)?;
    validate_env_config(&config, &keys).await?;

    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    // valance validation
    balance_validation::balance_validation(&mut state, mode, &config, &keys).await?;

    // wait for prover to be ready
    println!(); // newline because print_status clears the last line
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
        RunMode::Config => unreachable!(),
    }

    Ok(())
}
