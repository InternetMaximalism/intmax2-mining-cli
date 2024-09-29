use availability::check_avaliability;
use configure::recover_withdrawal_private_key;
use console::{initialize_console, print_status};

use crate::{
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::{env_config::EnvConfig, env_validation::validate_env_config, errors::CLIError},
};

pub mod accounts_status;
pub mod availability;
pub mod balance_validation;
pub mod configure;
pub mod console;
pub mod export_deposit_accounts;
pub mod interactive;

pub async fn run(mode: RunMode) -> Result<(), CLIError> {
    check_avaliability()
        .await
        .map_err(|e| CLIError::VersionError(e.to_string()))?;
    let mode = if mode == RunMode::Interactive {
        interactive::interactive()
            .await
            .map_err(|e| CLIError::InternalError(e.to_string()))?
    } else {
        mode.clone()
    };

    let config = EnvConfig::import_from_env().map_err(|e| CLIError::EnvError(e.to_string()))?;
    let withdrawal_private_key = recover_withdrawal_private_key(&config)
        .map_err(|e| CLIError::InternalError(e.to_string()))?;

    validate_env_config(&config)
        .await
        .map_err(|e| CLIError::EnvError(e.to_string()))?;
    config
        .export_to_env()
        .map_err(|e| CLIError::EnvError(e.to_string()))?;

    // for export mode, we only need to export the deposit accounts and exit
    if mode == RunMode::Export {
        export_deposit_accounts::export_deposit_accounts(withdrawal_private_key)
            .await
            .map_err(|e| CLIError::InternalError(e.to_string()))?;
        return Ok(());
    }

    let mut state = State::new();
    let prover_future = tokio::spawn(async { Prover::new() });

    let start_key_number =
        accounts_status::accounts_status(&mut state, config.mining_times, withdrawal_private_key)
            .await
            .map_err(|e| CLIError::InternalError(e.to_string()))?;

    // wait for prover to be ready
    initialize_console();
    print_status("Waiting for prover to be ready");
    let prover = prover_future
        .await
        .map_err(|e| CLIError::InternalError(e.to_string()))?;
    state.prover = Some(prover);

    match mode {
        RunMode::Mining => {
            mining_loop(
                &mut state,
                withdrawal_private_key,
                start_key_number,
                config.mining_unit,
                config.mining_times,
            )
            .await
            .map_err(|e| CLIError::InternalError(e.to_string()))?;
        }
        RunMode::Claim => {
            claim_loop(&mut state, withdrawal_private_key)
                .await
                .map_err(|e| CLIError::InternalError(e.to_string()))?;
        }
        RunMode::Exit => {
            exit_loop(&mut state, withdrawal_private_key)
                .await
                .map_err(|e| CLIError::InternalError(e.to_string()))?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
