use std::collections::HashMap;

use intmax2_zkp::ethereum_types::bytes32::Bytes32;
use mining_circuit::eligible_tree::{EligibleLeaf, EligibleTree, ELIGIBLE_TREE_HEIGHT};

#[derive(Debug, Clone)]
pub struct EligibleTreeWithMap {
    pub tree: EligibleTree,
    pub map: HashMap<u32, u32>, // key: deposit_index, value: eligible index
}

impl EligibleTreeWithMap {
    pub fn new() -> Self {
        let tree = EligibleTree::new(ELIGIBLE_TREE_HEIGHT);
        let map = HashMap::new();
        Self { tree, map }
    }

    pub fn push(&mut self, leaf: EligibleLeaf) {
        let leaf_index = self.tree.len() as u32;
        self.map.insert(leaf.deposit_index, leaf_index);
        self.tree.push(leaf);
    }

    pub fn get_leaf_index(&self, deposit_index: u32) -> Option<u32> {
        self.map.get(&deposit_index).copied()
    }

    pub fn get_root(&self) -> Bytes32 {
        self.tree.get_root().into()
    }
}
