use claim::{
    claim_task::claim_task,
    determin::{determin_next_claim_process, ClaimProcess},
};
use mining::{
    cancel::cancel_task,
    deposit::deposit_task,
    determin::{determin_next_mining_process, MiningProcess},
    withdrawal::withdrawal_task,
};
use rand::Rng;

use crate::{cli::console::print_status, config::Settings, state::state::State};

pub mod claim;
pub mod mining;
pub mod sync;

pub async fn main_loop(state: &mut State) -> anyhow::Result<()> {
    let mut is_mining_ended = false;

    loop {
        state.sync_trees().await?;
        if is_mining_ended {
            break;
        }
        let next_process = determin_next_mining_process(state).await?;
        match next_process {
            MiningProcess::Deposit => {
                print_status("Deposit");
                deposit_task(state).await?;
                mining_cooldown().await?;
            }
            MiningProcess::Withdrawal(event) => {
                print_status("Withdrawal");
                withdrawal_task(state, event).await?;
                mining_cooldown().await?;
            }
            MiningProcess::Cancel(event) => {
                print_status("Cancel");
                cancel_task(state, event).await?
            }
            MiningProcess::WaitingForAnalyze => {
                print_status("Waiting for analyze");
                main_loop_cooldown().await?;
                continue;
            }
            MiningProcess::EndBecauseOfNoRemainingDeposit => {
                print_status("Mining end");
                println!("No deposit is remaining. Please use new deposit address.");
                println!();
                is_mining_ended = true;
            }
            MiningProcess::EndBecauseOfShutdown => {
                print_status("No withdrawal is remaining");
                is_mining_ended = true;
            }
        }
        let next_process = determin_next_claim_process(state).await?;
        match next_process {
            ClaimProcess::Claim(events) => {
                print_status(&format!("Claim {}", events.len()));
                claim_task(state, &events).await?;
                claim_cooldown().await?;
            }
            ClaimProcess::Wait => (),
            ClaimProcess::End => break,
        }
        main_loop_cooldown().await?;
    }

    println!("Mining and Claim process ended.");

    Ok(())
}

/// Cooldown for main loop. `main_loop_cooldown_in_sec` seconds.
/// To avoid spamming RPC calls.
async fn main_loop_cooldown() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    tokio::time::sleep(std::time::Duration::from_secs(
        settings.service.main_loop_cooldown_in_sec,
    ))
    .await;
    Ok(())
}

/// Cooldown for mining. Random time between 0 and `mining_max_cooldown_in_sec` to improve privacy.
async fn mining_cooldown() -> anyhow::Result<()> {
    print_status("Mining cooldown...");
    let settings = Settings::new()?;
    let cooldown = rand::thread_rng().gen_range(0..settings.service.mining_max_cooldown_in_sec);
    tokio::time::sleep(std::time::Duration::from_secs(cooldown)).await;
    Ok(())
}

/// Cooldown for claim. Random time between 0 and `claim_max_cooldown_in_sec` to improve privacy.
async fn claim_cooldown() -> anyhow::Result<()> {
    print_status("Claim cooldown...");
    let settings = Settings::new()?;
    let cooldown = rand::thread_rng().gen_range(0..settings.service.claim_max_cooldown_in_sec);
    tokio::time::sleep(std::time::Duration::from_secs(cooldown)).await;
    Ok(())
}
