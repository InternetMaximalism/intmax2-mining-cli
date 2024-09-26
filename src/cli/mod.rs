use availability::check_avaliability;
use console::print_status;
use load_env::Config;

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
    println!(
        "Welcome to the INTMAX mining CLI!. Network: {}, Mode: {:?}",
        get_network(),
        mode
    );
    check_avaliability().await?;

    let config = Config::load().await?;
    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    // valance validation
    balance_validation::balance_validation(&mut state, mode, config.clone()).await?;

    // wait for prover to be ready
    println!(); // newline because print_status clears the last line
    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    // main loop
    match mode {
        RunMode::Mining => {
            mining_loop(
                &mut state,
                &config.keys,
                config.mining_unit,
                config.mining_times,
            )
            .await?
        }
        RunMode::Claim => claim_loop(&mut state, &config.keys).await?,
        RunMode::Exit => exit_loop(&mut state, &config.keys).await?,
        RunMode::Config => unreachable!(),
    }

    Ok(())
}
