use console::print_status;
use load_env::{load_env, Config};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::network::get_network,
};

pub mod availability;
pub mod console;
pub mod load_env;
pub mod private_data;
// pub mod status;
pub mod balance_validation;
// pub mod user_settings;

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    let config = load_env(mode).await?;

    // start up
    // let mut state = start(mode).await?;

    let mut state = State::new();
    println!("Welcome to the INTMAX mining CLI!");
    println!("Network: {}", get_network());
    let prover_future = tokio::spawn(async { Prover::new() });

    // resume task
    // print_status("Checking for pending tasks");
    // resume_withdrawal_task(&state).await?;
    // resume_claim_task(&state).await?;

    // validation

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

// async fn start(mode: RunMode) -> anyhow::Result<State> {

//     // check availability
//     check_avaliability().await?;

//     // private settings
//     let private_data = private_data::set_private_data().await?;

//     // construct state
//     let mut state = State::new(private_data.clone(), mode);

//     // user settings
//     user_settings::user_settings(&private_data).await?;

//     // print status
//     print_cli_status(&mut state, &private_data).await?;

//     print_status("Waiting for prover to be ready");
//     let prover = prover_future.await?;
//     state.prover = Some(prover);

//     Ok(state)
// }
