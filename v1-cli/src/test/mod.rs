use chrono::NaiveDateTime;
use intmax2_zkp::ethereum_types::u256::U256;
use mining_circuit::eligible_tree::EligibleLeaf;
use num_bigint::BigUint;

use crate::{
    private_data::PrivateData,
    state::state::{RunMode, State},
    utils::{deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap},
};

pub fn get_dummy_state() -> State {
    let private_data = PrivateData::new(
        "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
        "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e",
        "0x8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199",
    )
    .unwrap();

    let mut eligible_tree = EligibleTreeWithMap::new();
    for i in 0..100 {
        eligible_tree.push(EligibleLeaf {
            deposit_index: i,
            amount: U256::try_from(BigUint::from(10u32).pow(18)).unwrap(),
        });
    }

    let state = State {
        private_data,
        deposit_hash_tree: DepositHashTree::new(),
        eligible_tree,
        last_tree_feched_at: NaiveDateTime::default(),
        last_deposit_synced_block: 0,
        mode: RunMode::Normal,
        prover: None,
    };
    state
}
