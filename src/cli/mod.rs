use availability::check_avaliability;
use configure::recover_withdrawal_private_key;
use console::{initialize_console, print_status};
use interactive::select_mode;

use crate::{
    external_api::contracts::utils::get_address,
    services::{claim_loop, exit_loop, mining_loop},
    state::{mode::RunMode, prover::Prover, state::State},
    utils::{env_config::EnvConfig, env_validation::validate_env_config, update},
};

pub mod accounts_status;
pub mod availability;
pub mod balance_validation;
pub mod configure;
pub mod console;
pub mod export_deposit_accounts;
pub mod interactive;

pub async fn run(mode: Option<RunMode>) -> anyhow::Result<()> {
    let is_interactive = mode.is_none();

    check_avaliability().await?;
    if is_interactive {
        interactive::interactive().await?;
    }

    let mut mode = if is_interactive {
        select_mode()?
    } else {
        mode.unwrap()
    };

    let config = EnvConfig::import_from_env()?;
    let withdrawal_private_key = recover_withdrawal_private_key(&config)?;
    if config.withdrawal_address != get_address(withdrawal_private_key) {
        anyhow::bail!("Withdrawal address does not match the address derived from the private key");
    }

    validate_env_config(&config).await?;
    config.export_to_env()?;

    let mut state = State::new();
    accounts_status::accounts_status(&mut state, config.mining_times, withdrawal_private_key)
        .await?;

    initialize_console();
    loop {
        match mode {
            RunMode::Mining => {
                initialize_prover(&mut state).await?;
                mining_loop(
                    &mut state,
                    withdrawal_private_key,
                    config.mining_unit,
                    config.mining_times,
                )
                .await?;
            }
            RunMode::Claim => {
                initialize_prover(&mut state).await?;
                claim_loop(&mut state, withdrawal_private_key).await?;
                press_any_key_to_continue().await;
            }
            RunMode::Exit => {
                initialize_prover(&mut state).await?;
                exit_loop(&mut state, withdrawal_private_key).await?;
                press_any_key_to_continue().await;
            }
            RunMode::Export => {
                export_deposit_accounts::export_deposit_accounts(withdrawal_private_key).await?;
                press_any_key_to_continue().await;
            }
            RunMode::CheckUpdate => {
                update::update()?;
                press_any_key_to_continue().await;
            }
        };
        if !is_interactive {
            // if not in interactive mode, we only run once
            break;
        }
        mode = select_mode()?;
    }
    Ok(())
}

async fn initialize_prover(state: &mut State) -> anyhow::Result<()> {
    if state.prover.is_none() {
        print_status("Waiting for prover to be ready");
        let prover = Prover::new();
        state.prover = Some(prover);
    }
    Ok(())
}

async fn press_any_key_to_continue() {
    println!("Press any key to continue...");
    let _ = tokio::io::AsyncReadExt::read(&mut tokio::io::stdin(), &mut [0u8]).await;
}
