use alloy::primitives::Address;
use reqwest::Client;

use crate::external_api::contracts::{
    events::{DepositLeafInserted, Deposited},
    utils::NormalProvider,
};

use super::error::GraphClientError;

// A wrapper around TheGraphClient that provides additional functionality for interacting with the L1 and L2 providers.
#[derive(Clone, Debug)]
pub struct GraphClient {
    pub client: Client,
    pub provider: NormalProvider,
}

impl GraphClient {
    // get all deposited events by sender address
    pub async fn get_deposited_event_by_sender(
        &self,
        _deposit_address: Address,
    ) -> Result<Vec<Deposited>, GraphClientError> {
        // todo!
        Ok(vec![])
    }

    pub async fn get_last_processed_deposit_id(
        &self,
        _deposit_address: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo!
        Ok(None)
    }

    pub async fn get_deposit_leaf_inserted_event(
        &self,
        _from_block: u64,
    ) -> Result<Vec<DepositLeafInserted>, GraphClientError> {
        // todo! fetch from the graph
        Ok(vec![])
    }

    pub async fn get_latest_deposit_timestamp(
        &self,
        _sender: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo! fetch from the graph
        Ok(None)
    }

    pub async fn get_latest_withdrawal_timestamp(
        &self,
        _recipient: Address,
    ) -> Result<Option<u64>, GraphClientError> {
        // todo! fetch from the graph
        Ok(None)
    }
}
