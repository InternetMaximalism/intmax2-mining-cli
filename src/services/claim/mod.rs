use claim::single_claim_task;

use crate::{
    state::{keys::Key, state::State},
    utils::errors::CLIError,
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
                .map_err(|e| CLIError::InternalError(format!("Failed to claim: {:#}", e)))?;
        }
    }
    Ok(())
}
