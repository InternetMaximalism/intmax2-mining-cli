use anyhow::Context;
use claim::single_claim_task;
use rand::Rng as _;

use crate::{
    cli::console::print_status,
    state::{keys::Key, state::State},
    utils::config::Settings,
};

use super::assets_status::AssetsStatus;

pub mod claim;
pub mod contract;
pub mod temp;
pub mod witness_generation;

pub const MAX_CLAIMS: usize = 10;

pub async fn claim_task(
    state: &State,
    key: &Key,
    assets_status: &AssetsStatus,
) -> anyhow::Result<()> {
    if !assets_status.not_claimed_indices.is_empty() {
        for events in assets_status.get_not_claimed_events().chunks(MAX_CLAIMS) {
            single_claim_task(state, key, &events)
                .await
                .context("Failed claim task")?;
        }
        claim_cooldown().await?;
    }
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
