use alloy::primitives::B256;
use chrono::NaiveDateTime;
use intmax2_zkp::ethereum_types::u256::U256;
use mining_circuit_v1::eligible_tree::EligibleLeaf;
use num_bigint::BigUint;

use crate::{
    external_api::contracts::{
        int1::Int1Contract,
        minter::MinterContract,
        token::TokenContract,
        utils::{get_address_from_private_key, get_provider},
    },
    state::{key::Key, prover::Prover, state::State},
    utils::{deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap},
};

pub fn get_dummy_keys() -> Key {
    let deposit_private_key: B256 =
        "0xdf57089febbacf7ba0bc227dafbffa9fc08a93fdc68e1e42411a14efcf23656e"
            .parse()
            .unwrap();
    let deposit_address = get_address_from_private_key(deposit_private_key);
    Key {
        deposit_private_key,
        deposit_address,
        withdrawal_address: deposit_address,
        withdrawal_private_key: deposit_private_key,
    }
}

pub async fn get_dummy_state(rpc_url: &str) -> State {
    let mut eligible_tree = EligibleTreeWithMap::new();
    for i in 0..100 {
        eligible_tree.push(EligibleLeaf {
            deposit_index: i,
            amount: U256::try_from(BigUint::from(10u32).pow(18)).unwrap(),
        });
    }
    let settings = crate::utils::config::Settings::load().unwrap();
    let provider = get_provider(rpc_url).unwrap();
    let int1 = Int1Contract::new(
        provider.clone(),
        settings.blockchain.int1_address.parse().unwrap(),
    );
    let minter = MinterContract::new(
        provider.clone(),
        settings.blockchain.minter_address.parse().unwrap(),
    );
    let token = TokenContract::new(
        provider.clone(),
        settings.blockchain.token_address.parse().unwrap(),
    );

    State {
        deposit_hash_tree: DepositHashTree::new(),
        short_term_eligible_tree: eligible_tree.clone(),
        long_term_eligible_tree: eligible_tree.clone(),
        last_tree_fetched_at: NaiveDateTime::default(),
        prover: Prover::new(),
        int1,
        minter,
        token,
        provider,
    }
}
