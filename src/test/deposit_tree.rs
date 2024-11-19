use intmax2_zkp::common::trees::deposit_tree;

use crate::{
    services::sync::sync_to_latest_deposit_tree,
    utils::{
        bin_parser::{BinDepositTree, DepositTreeInfo},
        config::Settings,
        deposit_hash_tree::DepositHashTree,
    },
};
use std::fs;

#[test]
fn test_load_deposit_tree_bin() {
    let bytes = fs::read("data/depositTree").unwrap();
    let tree: BinDepositTree = bincode::deserialize(&bytes).unwrap();
    let tree_info: DepositTreeInfo = tree.try_into().unwrap();

    println!("root hash {}", tree_info.root);

    let mut deposit_tree = DepositHashTree::new();
    sync_to_latest_deposit_tree(&mut deposit_tree, tree_info);
}

fn fetch_deposit_tree() -> DepositHashTree {
    todo!()
}
