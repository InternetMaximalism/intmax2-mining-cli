use availability::check_avaliability;
use console::print_status;
use dialoguer::Select;
use status::print_cli_status;

use crate::{
    services::{
        claim::claim::resume_claim_task, main_loop, mining::withdrawal::resume_withdrawal_task,
    },
    state::{mode::RunMode, prover::Prover, state::State},
    utils::network::get_network,
};

pub mod availability;
pub mod console;
pub mod private_data;
pub mod status;
pub mod user_settings;

pub async fn run() -> anyhow::Result<()> {
    // start up
    let mut state = start().await?;

    // resume task
    print_status("Checking for pending tasks");
    resume_withdrawal_task(&state).await?;
    resume_claim_task(&state).await?;

    // main loop
    main_loop(&mut state).await?;

    Ok(())
}

async fn start() -> anyhow::Result<State> {
    println!("Welcome to the INTMAX mining CLI!");
    println!("Network: {}", get_network());
    let prover_future = tokio::spawn(async { Prover::new() });

    // check availability
    check_avaliability().await?;

    // private settings
    let private_data = private_data::set_private_data().await?;

    // construct state
    let mode = select_mode();
    let mut state = State::new(private_data.clone(), mode);

    // user settings
    user_settings::user_settings(&private_data).await?;

    // print status
    print_cli_status(&mut state, &private_data).await?;

    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    Ok(state)
}

fn select_mode() -> RunMode {
    let items = vec!["Mining", "Claim", "Exit", "Wait for Claim"];
    let selection = Select::new()
        .with_prompt("Choose mode")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();
    match selection {
        0 => RunMode::Mining,
        1 => RunMode::Claim,
        2 => RunMode::Exit,
        3 => RunMode::WaitForClaim,
        _ => unreachable!(),
    }
}
