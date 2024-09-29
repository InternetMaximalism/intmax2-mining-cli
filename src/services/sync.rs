use crate::{
    external_api::{
        contracts::{
            events::get_deposit_leaf_inserted_event,
            int1::{get_deposit_root, get_deposit_root_exits},
        },
        github::fetch_latest_tree_from_github,
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

const MAX_TRY_FETCH_TREE: usize = 10;

pub async fn sync_trees(
    last_deposit_block_number: &mut u64,
    last_update: &mut NaiveDateTime,
    deposit_hash_tree: &mut DepositHashTree,
    eligible_tree: &mut EligibleTreeWithMap,
) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    let sync_tree_data_interval_in_sec = Settings::load()?.api.sync_tree_data_interval_in_sec;
    if now.signed_duration_since(*last_update)
        <= chrono::Duration::seconds(sync_tree_data_interval_in_sec as i64)
    {
        // sync deposit tree only
        *last_deposit_block_number =
            sync_to_latest_deposit_tree(deposit_hash_tree, *last_deposit_block_number).await?;
        info!(
            "No need to fetch latest trees from GitHub, last update: {}, deposit_len: {}, eligible_len: {}, last deposit block number: {}",
            last_update, deposit_hash_tree.tree.len(), eligible_tree.tree.len(), last_deposit_block_number
        );
        return Ok(());
    }
    let mut try_number = 0;
    loop {
        if try_number > MAX_TRY_FETCH_TREE {
            anyhow::bail!("Exceeded MAX_TRY_FETCH_TREE");
        }
        let result = fetch_latest_tree_from_github(*last_update).await?;
        if let Some((bin_deposit_tree, bin_eligible_tree, new_last_update)) = result {
            // in the case that new trees found in github
            match validate_fetched_tree(bin_deposit_tree, bin_eligible_tree).await {
                // in the case that the fetched tree is valid
                Ok((deposit_tree_info, eligible_tree_info)) => {
                    *last_update = new_last_update;
                    *deposit_hash_tree = deposit_tree_info.tree;
                    *eligible_tree = eligible_tree_info.tree;

                    info!(
                "Fetched latest trees from GitHub, last update: {}, deposit_len: {}, deposit_root: {}, eligible_len: {}, eligible_root: {}, last deposit block number: {}",
                last_update, deposit_hash_tree.tree.len(), deposit_hash_tree.get_root(),  eligible_tree.tree.len(),eligible_tree.get_root(), last_deposit_block_number
            );
                    break;
                }
                // in the case that the fetched tree is invalid
                Err(e) => {
                    warn!("Feched tree is invalid in try {}: {}", try_number, e);
                    // retry after sleep
                    sleep(std::time::Duration::from_secs(30)).await;
                    try_number += 1;
                    continue;
                }
            }
        } else {
            // in the case that new trees are not found.
            *last_deposit_block_number =
                sync_to_latest_deposit_tree(deposit_hash_tree, *last_deposit_block_number).await?;
            *last_update = now; // update last_update to now
            info!(
                "No new trees found on GitHub, last update: {}, deposit_len: {}, eligible_len: {}, last deposit block number: {}",
                last_update, deposit_hash_tree.tree.len(), eligible_tree.tree.len(), last_deposit_block_number
            );
            break;
        }
    }
    Ok(())
}

async fn validate_fetched_tree(
    bin_deposit_tree: BinDepositTree,
    bin_eligible_tree: BinEligibleTree,
) -> anyhow::Result<(DepositTreeInfo, EligibleTreeInfo)> {
    let deposit_tree_info: DepositTreeInfo = bin_deposit_tree
        .try_into()
        .map_err(|e| anyhow::anyhow!("deposit tree deseiarize error {}", e))?;
    let eligible_tree_info: EligibleTreeInfo = bin_eligible_tree
        .try_into()
        .map_err(|e| anyhow::anyhow!("eligible tree deseiarize error {}", e))?;
    // check roots
    let deposit_root_exists = get_deposit_root_exits(deposit_tree_info.root).await?;
    ensure!(
        deposit_root_exists,
        "Deposit root does not exist on chain: {}",
        deposit_tree_info.root
    );
    let onchain_eligible_root = crate::external_api::contracts::minter::get_eligible_root().await?;
    ensure!(
        onchain_eligible_root == eligible_tree_info.root,
        "Eligible tree rood does not match"
    );
    Ok((deposit_tree_info, eligible_tree_info))
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
    async fn sync_to_latest_deposit_tree() {
        let mut deposit_hash_tree = DepositHashTree::new();
        let mut eligible_tree = EligibleTreeWithMap::new();
        let mut last_deposit_block_number = 0;
        let mut last_update = chrono::NaiveDateTime::default();
        super::sync_trees(
            &mut last_deposit_block_number,
            &mut last_update,
            &mut deposit_hash_tree,
            &mut eligible_tree,
        )
        .await
        .unwrap();

        dbg!(deposit_hash_tree.tree.len());
    }
}
