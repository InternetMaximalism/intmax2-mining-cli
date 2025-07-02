use chrono::NaiveDateTime;

use super::{key::Key, prover::Prover};
use crate::{
    external_api::contracts::{
        int1::Int1Contract,
        minter::MinterContract,
        token::TokenContract,
        utils::{get_provider, NormalProvider},
    },
    services::{
        assets_status::{fetch_assets_status, AssetsStatus},
        sync::sync_trees,
    },
    utils::{
        config::Settings, deposit_hash_tree::DepositHashTree,
        eligible_tree_with_map::EligibleTreeWithMap,
    },
};

pub struct State {
    pub deposit_hash_tree: DepositHashTree,
    pub short_term_eligible_tree: EligibleTreeWithMap,
    pub long_term_eligible_tree: EligibleTreeWithMap,
    pub last_tree_fetched_at: NaiveDateTime,
    pub prover: Prover,

    // block chain state
    pub int1: Int1Contract,
    pub minter: MinterContract,
    pub token: TokenContract,
    pub provider: NormalProvider,
}

impl State {
    pub fn new(rpc_url: &str) -> Self {
        let settings = Settings::load().unwrap();
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

        Self {
            deposit_hash_tree: DepositHashTree::new(),
            short_term_eligible_tree: EligibleTreeWithMap::new(),
            long_term_eligible_tree: EligibleTreeWithMap::new(),
            last_tree_fetched_at: NaiveDateTime::default(),
            prover: Prover::new(),
            int1,
            minter,
            token,
            provider,
        }
    }

    pub async fn sync_trees(&mut self) -> anyhow::Result<()> {
        sync_trees(
            &self.int1,
            &self.minter,
            &mut self.last_tree_fetched_at,
            &mut self.deposit_hash_tree,
            &mut self.short_term_eligible_tree,
            &mut self.long_term_eligible_tree,
        )
        .await?;
        Ok(())
    }

    pub async fn sync_and_fetch_assets(&mut self, key: &Key) -> anyhow::Result<AssetsStatus> {
        self.sync_trees().await?;
        fetch_assets_status(self, key.deposit_address, key.deposit_private_key).await
    }
}
