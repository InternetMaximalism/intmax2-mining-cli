use anyhow::ensure;
use intmax2_zkp::{
    constants::DEPOSIT_TREE_HEIGHT,
    ethereum_types::{bytes32::Bytes32, u256::U256, u32limb_trait::U32LimbTrait},
};
use mining_circuit_v1::eligible_tree::{EligibleLeaf, ELIGIBLE_TREE_HEIGHT};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use super::{deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BinEligibleLeaf {
    pub deposit_index: u32,
    pub amount: [u8; 32],
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BinEligibleTree {
    pub root_hash: [u8; 32],
    pub block_number: u64,
    pub tree_height: u32,
    pub leaves: Vec<BinEligibleLeaf>,
}

#[derive(Clone)]
pub struct EligibleTreeInfo {
    pub root: Bytes32,
    pub block_number: u64,
    pub tree: EligibleTreeWithMap,
}

impl TryFrom<BinEligibleTree> for EligibleTreeInfo {
    type Error = anyhow::Error;

    fn try_from(bin_tree: BinEligibleTree) -> anyhow::Result<Self> {
        let mut tree = EligibleTreeWithMap::new();
        for leaf in bin_tree.leaves {
            let amount: U256 = BigUint::from_bytes_le(&leaf.amount).try_into()?;
            tree.push(EligibleLeaf {
                deposit_index: leaf.deposit_index,
                amount,
            });
        }
        let expected_root = Bytes32::from_bytes_be(&bin_tree.root_hash);
        let actual_root: Bytes32 = tree.get_root().try_into()?;
        ensure!(
            actual_root == expected_root,
            "Root hash mismatch: expected {}, got {}",
            expected_root,
            tree.get_root()
        );
        Ok(Self {
            root: actual_root,
            block_number: bin_tree.block_number,
            tree,
        })
    }
}

impl From<EligibleTreeInfo> for BinEligibleTree {
    fn from(tree_info: EligibleTreeInfo) -> Self {
        let leaves: Vec<BinEligibleLeaf> = tree_info
            .tree
            .tree
            .leaves()
            .iter()
            .map(|leaf| BinEligibleLeaf {
                deposit_index: leaf.deposit_index,
                amount: leaf
                    .amount
                    .to_bytes_be()
                    .into_iter()
                    .rev()
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap(),
            })
            .collect();
        Self {
            root_hash: tree_info.root.to_bytes_be().try_into().unwrap(),
            block_number: tree_info.block_number,
            tree_height: ELIGIBLE_TREE_HEIGHT as u32,
            leaves,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BinDepositTree {
    pub root_hash: [u8; 32],
    pub block_number: u64,
    pub tree_height: u32,
    pub leaf_hashes: Vec<[u8; 32]>,
}

pub struct DepositTreeInfo {
    pub root: Bytes32,
    pub block_number: u64,
    pub tree: DepositHashTree,
}

impl TryFrom<BinDepositTree> for DepositTreeInfo {
    type Error = anyhow::Error;

    fn try_from(bin_tree: BinDepositTree) -> anyhow::Result<Self> {
        let mut tree = DepositHashTree::new();
        for leaf_hash in bin_tree.leaf_hashes {
            let leaf_hash: Bytes32 = Bytes32::from_bytes_be(&leaf_hash);
            tree.push(leaf_hash);
        }
        let expected_root = Bytes32::from_bytes_be(&bin_tree.root_hash);
        let actual_root: Bytes32 = tree.get_root().try_into()?;
        ensure!(
            actual_root == expected_root,
            "Root hash mismatch: expected {}, got {}",
            expected_root,
            tree.get_root()
        );
        ensure!(bin_tree.tree_height == DEPOSIT_TREE_HEIGHT as u32);
        Ok(Self {
            root: actual_root,
            block_number: bin_tree.block_number,
            tree,
        })
    }
}

impl From<DepositTreeInfo> for BinDepositTree {
    fn from(tree_info: DepositTreeInfo) -> Self {
        let leaf_hashes: Vec<[u8; 32]> = tree_info
            .tree
            .tree
            .leaves()
            .iter()
            .map(|leaf_hash| {
                let leaf_hash: [u8; 32] = leaf_hash.0.to_bytes_be().try_into().unwrap();
                leaf_hash
            })
            .collect();
        Self {
            root_hash: tree_info.root.to_bytes_be().try_into().unwrap(),
            block_number: tree_info.block_number,
            tree_height: DEPOSIT_TREE_HEIGHT as u32,
            leaf_hashes,
        }
    }
}
