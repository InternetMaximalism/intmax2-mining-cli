use availability::check_avaliability;
use configure::recover_withdrawal_private_key;
use console::{initialize_console, print_status};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::{env_config::EnvConfig, env_validation::validate_env_config},
};

pub mod accounts_status;
pub mod availability;
pub mod balance_validation;
pub mod configure;
pub mod console;
pub mod export_deposit_accounts;
pub mod interactive;

pub async fn run(mode: RunMode) -> anyhow::Result<()> {
    check_avaliability().await?;
    let mode = if mode == RunMode::Interactive {
        interactive::interactive().await?
    } else {
        mode.clone()
    };

    let config = EnvConfig::import_from_env()?;
    let withdrawal_private_key = recover_withdrawal_private_key(&config)?;

    validate_env_config(&config).await?;
    config.export_to_env()?;

    // for export mode, we only need to export the deposit accounts and exit
    if mode == RunMode::Export {
        export_deposit_accounts::export_deposit_accounts(withdrawal_private_key).await?;
        return Ok(());
    }

    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    accounts_status::accounts_status(&mut state, config.mining_times, withdrawal_private_key)
        .await?;

    // wait for prover to be ready
    initialize_console();
    print_status("Waiting for prover to be ready");
    let prover = prover_future.await?;
    state.prover = Some(prover);

    match mode {
        RunMode::Mining => {
            mining_loop(
                &mut state,
                withdrawal_private_key,
                config.mining_unit,
                config.mining_times,
            )
            .await?;
        }
        RunMode::Claim => {
            claim_loop(&mut state, withdrawal_private_key).await?;
        }
        RunMode::Exit => {
            exit_loop(&mut state, withdrawal_private_key).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
