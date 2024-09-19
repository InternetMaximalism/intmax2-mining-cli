use std::collections::HashMap;

use intmax2_zkp::{
    common::{deposit::Deposit, trees::deposit_tree::DepositMerkleProof},
    constants::DEPOSIT_TREE_HEIGHT,
    ethereum_types::bytes32::Bytes32,
    utils::{
        leafable::Leafable,
        leafable_hasher::KeccakLeafableHasher,
        trees::{
            incremental_merkle_tree::{IncrementalMerkleProof, IncrementalMerkleTree},
            merkle_tree::MerkleProof,
        },
    },
};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DepositHash(pub Bytes32);

impl Leafable for DepositHash {
    type LeafableHasher = KeccakLeafableHasher;

    fn empty_leaf() -> Self {
        DepositHash(Deposit::default().hash())
    }

    fn hash(&self) -> Bytes32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct DepositHashTree {
    pub tree: IncrementalMerkleTree<DepositHash>,
    pub hashes: HashMap<Bytes32, u32>,
}

impl DepositHashTree {
    pub fn new() -> Self {
        Self {
            tree: IncrementalMerkleTree::new(DEPOSIT_TREE_HEIGHT),
            hashes: HashMap::new(),
        }
    }

    pub fn get_root(&self) -> Bytes32 {
        self.tree.get_root()
    }

    pub fn push(&mut self, hash: Bytes32) {
        let index = self.tree.len();
        self.tree.push(DepositHash(hash));
        self.hashes.insert(hash, index as u32);
    }

    pub fn contains(&self, hash: Bytes32) -> bool {
        self.hashes.contains_key(&hash)
    }

    pub fn get_index(&self, hash: Bytes32) -> Option<u32> {
        self.hashes.get(&hash).copied()
    }

    pub fn prove(&self, index: u32) -> DepositMerkleProof {
        let proof = self.tree.prove(index as usize);
        IncrementalMerkleProof(MerkleProof {
            siblings: proof.0.siblings,
        })
    }
}

#[cfg(test)]
mod tests {
    use intmax2_zkp::{
        common::{deposit::Deposit, trees::deposit_tree::DepositTree},
        constants::DEPOSIT_TREE_HEIGHT,
        utils::leafable::Leafable as _,
    };
    use rand::Rng as _;

    use super::DepositHashTree;

    #[test]
    fn test_deposit_hash_tree() {
        let mut rng = rand::thread_rng();
        let n = 10;

        let mut deposit_tree = DepositTree::new(DEPOSIT_TREE_HEIGHT);
        for _ in 0..n {
            deposit_tree.push(Deposit::rand(&mut rng));
        }
        let root = deposit_tree.get_root();

        let mut deposit_hash_tree = DepositHashTree::new();
        for deposit in &deposit_tree.leaves() {
            deposit_hash_tree.push(deposit.hash());
        }
        let root2 = deposit_hash_tree.tree.get_root();
        assert_eq!(root, root2);

        let rand_index = rng.gen_range(0..n);
        let deposit = deposit_tree.get_leaf(rand_index);
        let proof = deposit_hash_tree.prove(rand_index as u32);
        proof.verify(&deposit, rand_index, root).unwrap();
        let index = deposit_hash_tree.get_index(deposit.hash()).unwrap();
        assert_eq!(index, rand_index as u32);
    }
}
