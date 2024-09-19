use crate::{
    cli::console::print_status,
    external_api::{
        contracts::{events::get_deposit_leaf_inserted_event, int1::get_deposit_root},
        github::fetch_latest_tree_from_github,
    },
    utils::{
        bin_parser::{DepositTreeInfo, EligibleTreeInfo},
        deposit_hash_tree::DepositHashTree,
        eligible_tree_with_map::EligibleTreeWithMap,
    },
};
use anyhow::ensure;
use chrono::{Duration, NaiveDateTime, Utc};

pub async fn sync_trees(
    last_deposit_block_number: &mut u64,
    last_update: &mut NaiveDateTime,
    deposit_hash_tree: &mut DepositHashTree,
    eligible_tree: &mut EligibleTreeWithMap,
) -> anyhow::Result<()> {
    let now = Utc::now().naive_utc();
    if now.signed_duration_since(*last_update) <= Duration::hours(24) {
        // sync deposit tree only
        *last_deposit_block_number =
            sync_to_latest_deposit_tree(deposit_hash_tree, *last_deposit_block_number).await?;
        return Ok(());
    }
    print_status("Fetching latest trees from github...");
    match fetch_latest_tree_from_github(*last_update).await? {
        Some((bin_deposit_tree, bin_eligible_tree, new_last_update)) => {
            let deposit_tree_info: DepositTreeInfo = bin_deposit_tree.try_into()?;
            let eligible_tree_info: EligibleTreeInfo = bin_eligible_tree.try_into()?;
            *last_update = new_last_update;
            *deposit_hash_tree = deposit_tree_info.tree;
            *eligible_tree = eligible_tree_info.tree;
            *last_deposit_block_number =
                sync_to_latest_deposit_tree(deposit_hash_tree, deposit_tree_info.block_number)
                    .await?;
        }
        None => {
            *last_update = now; // update last_update to now
        }
    }
    Ok(())
}

async fn sync_to_latest_deposit_tree(
    deposit_hash_tree: &mut DepositHashTree,
    from_block: u64,
) -> anyhow::Result<u64> {
    let events = get_deposit_leaf_inserted_event(from_block).await?;
    let mut to_block_number = from_block;
    for event in events {
        if deposit_hash_tree.tree.len() == event.deposit_index as usize {
            deposit_hash_tree.push(event.deposit_hash);
            to_block_number = event.block_number;
        }
    }
    let expected_root = get_deposit_root().await?;
    let actual_root = deposit_hash_tree.get_root();
    ensure!(
        expected_root == actual_root,
        "Root mismatch: expected {}, got {}",
        expected_root,
        actual_root
    );
    Ok(to_block_number)
}
