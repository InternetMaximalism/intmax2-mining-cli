use console::print_status;
use load_env::{load_env, Config};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::network::get_network,
};

pub mod availability;
pub mod balance_validation;
pub mod console;
pub mod load_env;

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    let config: Config = load_env(mode).await?;

    let mut state = State::new();
    println!("Welcome to the INTMAX mining CLI!");
    println!("Network: {}", get_network());
    let prover_future = tokio::spawn(async { Prover::new() });

    // valance validation
    balance_validation::balance_validation(&mut state, config.clone()).await?;

    // wait for prover to be ready
    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    // main loop
    match mode {
        RunMode::Mining => match config {
            Config::Mining(mining_config) => {
                mining_loop(
                    &mut state,
                    &mining_config.keys,
                    mining_config.mining_unit,
                    mining_config.mining_times,
                )
                .await?
            }
            _ => unreachable!(),
        },
        RunMode::Claim => match config {
            Config::Claim(claim_config) => claim_loop(&mut state, claim_config.keys).await?,
            _ => unreachable!(),
        },
        RunMode::Exit => match config {
            Config::Exit(exit_config) => exit_loop(&mut state, &exit_config.keys).await?,
            _ => unreachable!(),
        },
    }

    Ok(())
}
