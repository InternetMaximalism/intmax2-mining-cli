use availability::check_avaliability;
use console::print_status;
use dialoguer::Password;

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{keys::Keys, mode::RunMode, prover::Prover, state::State},
    utils::{encryption::decrypt, env_config::EnvConfig, network::get_network},
};

pub mod availability;
pub mod balance_validation;
pub mod console;
// pub mod load_env;

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    println!(
        "Welcome to the INTMAX mining CLI!. Network: {}, Mode: {:?}",
        get_network(),
        mode
    );
    check_avaliability().await?;

    let config = EnvConfig::import_from_env()?;
    let keys = if let Some(keys) = &config.keys {
        keys.clone()
    } else {
        // require password to decrypt keys
        let password = Password::new().with_prompt("Password").interact()?;
        let keys: Keys = decrypt(&password, config.encrypted_keys.as_ref().unwrap())?;
        keys
    };

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
