use claim::single_claim_task;

use crate::{
    state::{key::Key, state::State},
    utils::errors::CLIError,
};

use super::assets_status::AssetsStatus;

pub mod claim;
pub mod contract;
pub mod temp;
pub mod witness_generation;

pub const MAX_CLAIMS: usize = 10;

pub async fn claim_task(
    state: &mut State,
    key: &Key,
    is_short_term: bool,
    assets_status: &AssetsStatus,
) -> anyhow::Result<()> {
    for events in assets_status
        .get_not_claimed_events(is_short_term)
        .chunks(MAX_CLAIMS)
    {
        single_claim_task(state, key, is_short_term, &events)
            .await
            .map_err(|e| CLIError::InternalError(format!("Failed to claim: {:#}", e)))?;
    }
    Ok(())
}
