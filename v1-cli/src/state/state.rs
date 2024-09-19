use chrono::NaiveDateTime;

use super::prover::Prover;
use crate::{
    private_data::PrivateData,
    services::sync::sync_trees,
    utils::{deposit_hash_tree::DepositHashTree, eligible_tree_with_map::EligibleTreeWithMap},
};

pub struct State {
    pub private_data: PrivateData,
    pub deposit_hash_tree: DepositHashTree,
    pub eligible_tree: EligibleTreeWithMap,
    pub last_tree_feched_at: NaiveDateTime,
    pub last_deposit_synced_block: u64,
    pub mode: RunMode,
    pub prover: Option<Prover>,
}

impl State {
    pub fn new(private_data: PrivateData, mode: RunMode) -> Self {
        Self {
            private_data,
            deposit_hash_tree: DepositHashTree::new(),
            eligible_tree: EligibleTreeWithMap::new(),
            last_tree_feched_at: NaiveDateTime::default(),
            last_deposit_synced_block: 0,
            mode,
            prover: None,
        }
    }

    pub fn build_circuit(&mut self) -> anyhow::Result<()> {
        self.prover = Some(Prover::new());
        Ok(())
    }

    pub async fn sync_trees(&mut self) -> anyhow::Result<()> {
        sync_trees(
            &mut self.last_deposit_synced_block,
            &mut self.last_tree_feched_at,
            &mut self.deposit_hash_tree,
            &mut self.eligible_tree,
        )
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Normal,
    Shutdown,
}
