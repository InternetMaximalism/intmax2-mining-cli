use crate::{
    external_api::{
        contracts::{
            events::get_deposit_leaf_inserted_event,
            int1::{get_deposit_root, get_deposit_root_exits},
        },
        github::{fetch_latest_tree_from_github, BinTrees},
    },
    utils::{
        bin_parser::{BinDepositTree, BinEligibleTree, DepositTreeInfo, EligibleTreeInfo},
        config::Settings,
        deposit_hash_tree::DepositHashTree,
        eligible_tree_with_map::EligibleTreeWithMap,
    },
};

use anyhow::ensure;
use chrono::{NaiveDateTime, Utc};
use log::{info, warn};
use tokio::time::sleep;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network Error: {}", _0)]
    NetworkError(String),
    #[error("Tree Deserialization Error {}", _0)]
    TreeDeserializationError(String),
    #[error("Tree Root Sync Error {}", _0)]
    TreeRootSyncError(String),
    #[error("Sync Deposit Tree From Events Error {}", _0)]
    SyncDepositTreeFromEventsError(String),
    #[error("Max Sync Trials Exceeded")]
    MaxSyncTrialsExceeded,
}

const MAX_TRY_FETCH_TREE: usize = 10;

pub async fn sync_trees(
    last_deposit_block_number: &mut u64,
    last_update: &mut NaiveDateTime,
    deposit_hash_tree: &mut DepositHashTree,
    short_term_eligible_tree: &mut EligibleTreeWithMap,
    long_term_eligible_tree: &mut EligibleTreeWithMap,
) -> Result<(), Error> {
    let sync_tree_data_interval_in_sec =
        Settings::load().unwrap().api.sync_tree_data_interval_in_sec;

    let now = Utc::now().naive_utc();

    // if last update is more than sync_tree_data_interval_in_sec, fetch latest trees from github
    if now.signed_duration_since(*last_update)
        > chrono::Duration::seconds(sync_tree_data_interval_in_sec as i64)
    {
        let mut try_number = 0;
        loop {
            if try_number > MAX_TRY_FETCH_TREE {
                return Err(Error::MaxSyncTrialsExceeded);
            }
            // fetch trees from github
            let BinTrees {
                bin_deposit_tree,
                bin_short_term_eligible_tree,
                bin_long_term_eligible_tree,
                latest_update: _,
            } = fetch_latest_tree_from_github(last_update.date())
                .await
                .map_err(|e| {
                    Error::NetworkError(format!("Failed to fetch latest tree from github: {}", e))
                })?;

            // retry if TreeRootSyncError occurs
            let update = || async {
                if let Some(bin_deposit_tree) = bin_deposit_tree {
                    *deposit_hash_tree =
                        parse_and_validate_bin_deposit_tree(bin_deposit_tree).await?;
                }
                if let Some(bin_short_term_eligible_tree) = bin_short_term_eligible_tree {
                    *short_term_eligible_tree =
                        parse_and_validate_bin_eligible_tree(true, bin_short_term_eligible_tree)
                            .await?;
                }
                if let Some(bin_long_term_eligible_tree) = bin_long_term_eligible_tree {
                    *long_term_eligible_tree =
                        parse_and_validate_bin_eligible_tree(false, bin_long_term_eligible_tree)
                            .await?;
                }
                Result::<(), Error>::Ok(())
            };
            match update().await {
                Ok(()) => break,
                Err(e) => {
                    warn!("Feched tree is invalid in try {}: {}", try_number, e);
                    try_number += 1;
                    sleep(std::time::Duration::from_secs(30)).await;
                }
            }
        }
    }
    // sync deposit tree only
    *last_deposit_block_number =
        sync_to_latest_deposit_tree(deposit_hash_tree, *last_deposit_block_number)
            .await
            .map_err(|e| {
                Error::SyncDepositTreeFromEventsError(format!("Failed to sync deposit tree: {}", e))
            })?;
    *last_update = now; // update last_update to now
    Ok(())
}

async fn parse_and_validate_bin_deposit_tree(
    bin_deposit_tree: BinDepositTree,
) -> Result<DepositHashTree, Error> {
    let deposit_tree_info: DepositTreeInfo = bin_deposit_tree
        .try_into()
        .map_err(|e: anyhow::Error| Error::TreeDeserializationError(e.to_string()))?;
    // check roots
    let deposit_root_exists = get_deposit_root_exits(deposit_tree_info.root)
        .await
        .map_err(|e| Error::NetworkError(format!("Failed to get deposit root: {}", e)))?;
    if !deposit_root_exists {
        return Err(Error::TreeRootSyncError(format!(
            "Deposit tree rood does not exist on chain: {}",
            deposit_tree_info.root
        )));
    }
    Ok(deposit_tree_info.tree)
}

async fn parse_and_validate_bin_eligible_tree(
    is_short_term: bool,
    bin_eligible_tree: BinEligibleTree,
) -> Result<EligibleTreeWithMap, Error> {
    let eligible_tree_info: EligibleTreeInfo = bin_eligible_tree
        .try_into()
        .map_err(|e: anyhow::Error| Error::TreeDeserializationError(e.to_string()))?;
    let onchain_eligible_root = if is_short_term {
        crate::external_api::contracts::minter::get_short_term_eligible_root()
            .await
            .map_err(|e| {
                Error::NetworkError(format!("Failed to get short term eligible root: {}", e))
            })?
    } else {
        crate::external_api::contracts::minter::get_long_term_eligible_root()
            .await
            .map_err(|e| {
                Error::NetworkError(format!("Failed to get long term eligible root: {}", e))
            })?
    };
    if onchain_eligible_root != eligible_tree_info.root {
        return Err(Error::TreeRootSyncError(format!(
            "Eligible tree rood does not match. Onchain: {:?}, Github {:?}",
            onchain_eligible_root, eligible_tree_info.root
        )));
    }
    Ok(eligible_tree_info.tree)
}

async fn sync_to_latest_deposit_tree(
    deposit_hash_tree: &mut DepositHashTree,
    from_block: u64,
) -> anyhow::Result<u64> {
    let events = get_deposit_leaf_inserted_event(from_block).await?;
    info!(
        "Syncing deposit tree from block {}, got {} events. Latest deposit_index={}",
        from_block,
        events.len(),
        events.last().map(|event| event.deposit_index).unwrap_or(0)
    );

    let next_deposit_index = deposit_hash_tree.tree.len();
    let mut to_append = events
        .iter()
        .filter(|event| event.deposit_index as usize >= next_deposit_index)
        .collect::<Vec<_>>();
    to_append.sort_by_key(|event| event.deposit_index);

    let mut to_block_number = from_block;
    for event in to_append {
        ensure!(
            event.deposit_index as usize == deposit_hash_tree.tree.len(),
            "Deposit index mismatch: expected {}, got {}",
            deposit_hash_tree.tree.len(),
            event.deposit_index
        );
        deposit_hash_tree.push(event.deposit_hash);
        to_block_number = event.block_number;
    }
    let local_root = deposit_hash_tree.get_root();
    let is_exists = get_deposit_root_exits(local_root).await?;
    ensure!(
        is_exists,
        "Local deposit root does not exist on chain: {}",
        local_root
    );
    let current_root = get_deposit_root().await?;

    // this may occur because of the delay of the event log
    if local_root != current_root {
        warn!(
            "Local deposit root mismatch: local {}, onchain {}.",
            local_root, current_root
        );
    }
    Ok(to_block_number)
}

#[cfg(test)]
mod tests {
    use crate::utils::{
        deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap,
    };

    #[tokio::test]
    #[ignore]
    async fn sync_to_latest_deposit_tree() {
        let mut deposit_hash_tree = DepositHashTree::new();
        let mut short_term_eligible_tree = EligibleTreeWithMap::new();
        let mut long_term_eligible_tree = EligibleTreeWithMap::new();
        let mut last_deposit_block_number = 0;
        let mut last_update = chrono::NaiveDateTime::default();
        super::sync_trees(
            &mut last_deposit_block_number,
            &mut last_update,
            &mut deposit_hash_tree,
            &mut short_term_eligible_tree,
            &mut long_term_eligible_tree,
        )
        .await
        .unwrap();

        dbg!(deposit_hash_tree.tree.len());
    }
}
