use std::io::{self, Read as _};

use ::console::{style, Term};
use alloy::primitives::B256;
use configure::recover_withdrawal_private_key;
use console::clear_console;
use mode_selection::{legacy_select_mode, select_mode};
use term_of_use::make_agreement;

use crate::{
    external_api::contracts::utils::get_address_from_private_key,
    services::{claim_loop, exit_loop, legacy_claim_loop, mining_loop},
    state::{mode::RunMode, state::State},
    utils::{
        env_config::EnvConfig,
        env_validation::validate_env_config,
        network::{get_network, is_legacy, Network},
        update,
    },
};

pub mod accounts_status;
pub mod availability;
pub mod balance_validation;
pub mod configure;
pub mod console;
pub mod export_deposit_accounts;
pub mod interactive;
pub mod mode_selection;
pub mod term_of_use;

pub async fn run(mode: Option<RunMode>) -> anyhow::Result<()> {
    make_agreement()?;

    let is_interactive = mode.is_none();

    if is_interactive {
        interactive::interactive().await?;
    }

    let config = EnvConfig::import_from_env()?;
    let withdrawal_private_key = recover_withdrawal_private_key(&config)?;
    if config.withdrawal_address != get_address_from_private_key(withdrawal_private_key) {
        anyhow::bail!("Withdrawal address does not match the address derived from the private key");
    }
    validate_env_config(&config).await?;
    config.export_to_env()?;

    if is_legacy() {
        print_legacy_warning();
        press_enter_to_continue();
    }

    let mut mode = if is_interactive {
        if is_legacy() {
            legacy_select_mode()?
        } else {
            select_mode()?
        }
    } else {
        mode.unwrap()
    };

    let mut state = State::new(&config.rpc_url);

    // prints the status of the accounts if mutable mode
    if mode == RunMode::Mining || mode == RunMode::Claim || mode == RunMode::Exit {
        accounts_status::accounts_status(&mut state, config.mining_times, withdrawal_private_key)
            .await?;
    }
    clear_console();
    mode_loop(
        &mut mode,
        &mut state,
        &config,
        withdrawal_private_key,
        is_interactive,
    )
    .await?;
    Ok(())
}

async fn mode_loop(
    mode: &mut RunMode,
    state: &mut State,
    config: &EnvConfig,
    withdrawal_private_key: B256,
    is_interactive: bool,
) -> anyhow::Result<()> {
    loop {
        match mode {
            RunMode::Mining => {
                mining_loop(
                    state,
                    withdrawal_private_key,
                    config.mining_unit,
                    config.mining_times,
                )
                .await?;
                press_enter_to_continue();
            }
            RunMode::Claim => {
                if is_legacy() {
                    legacy_claim_loop(state, withdrawal_private_key).await?;
                } else {
                    claim_loop(state, withdrawal_private_key).await?;
                }
                press_enter_to_continue();
            }
            RunMode::Exit => {
                exit_loop(state, withdrawal_private_key).await?;
                press_enter_to_continue();
            }
            RunMode::Export => {
                if is_legacy() {
                    export_deposit_accounts::legacy_export_deposit_accounts(
                        &state.provider,
                        withdrawal_private_key,
                    )
                    .await?;
                } else {
                    export_deposit_accounts::export_deposit_accounts(
                        &state.provider,
                        withdrawal_private_key,
                    )
                    .await?;
                }
                press_enter_to_continue();
            }
            RunMode::CheckUpdate => {
                update::update()?;
                press_enter_to_continue();
            }
        };
        if !is_interactive {
            // if not in interactive mode, we only run once
            break;
        }
        *mode = if is_legacy() {
            legacy_select_mode()?
        } else {
            select_mode()?
        };
    }
    Ok(())
}

pub fn press_enter_to_continue() {
    println!("Press Enter to continue...");
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer).unwrap();
}

pub fn print_legacy_warning() {
    let term = Term::stdout();
    let network = get_network();

    let colored_message = if network == Network::Mainnet {
        format!(
        "{} {}",
        style("WARNING:").yellow().bold(),
        style("Mining has transitioned from Mainnet to Base. Currently, on Mainnet, only asset withdrawals and token claims are possible.")
            .yellow()
    )
    } else {
        format!(
        "{} {}",
        style("WARNING:").yellow().bold(),
        style("Mining Testnet has transitioned from Holesky to Base-Sepolia. Currently, on Holesky, only asset withdrawals and token claims are possible.")
            .yellow())
    };
    term.write_line(&colored_message).unwrap();
}
