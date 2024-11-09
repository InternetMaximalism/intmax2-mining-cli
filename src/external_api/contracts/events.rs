use ethers::{providers::Middleware as _, types::Address};
use intmax2_zkp::{
    common::deposit::Deposit,
    ethereum_types::{bytes32::Bytes32, u256::U256, u32limb_trait::U32LimbTrait},
};
use log::info;

use crate::{external_api::contracts::utils::get_block, utils::retry::with_retry};

use super::{
    error::BlockchainError,
    int1::get_int1_contract,
    utils::{get_client, u256_as_bytes_be},
};

#[derive(Clone, Debug)]
pub struct Deposited {
    pub deposit_id: u64,
    pub sender: Address,
    pub recipient_salt_hash: Bytes32,
    pub token_index: u32,
    pub amount: U256,
    pub tx_nonce: u64,
}

impl Deposited {
    pub fn deposit(&self) -> Deposit {
        Deposit {
            pubkey_salt_hash: self.recipient_salt_hash,
            token_index: self.token_index,
            amount: self.amount,
        }
    }
}

pub async fn get_deposited_event_by_sender(
    sender: Address,
) -> Result<Vec<Deposited>, BlockchainError> {
    info!("get_deposited_event_by_sender");
    let int1 = get_int1_contract().await?;
    let events = with_retry(|| async {
        int1.deposited_filter()
            .from_block(0)
            .address(int1.address().into())
            .topic2(sender)
            .query_with_meta()
            .await
    })
    .await
    .map_err(|_| BlockchainError::NetworkError("failed to get deposited event".to_string()))?;
    let client = get_client().await?;
    let mut deposited_events = Vec::new();
    for (event, meta) in events {
        // get tx_nonce  because it is needed for getting deposit salt
        let tx_hash = meta.transaction_hash;
        let tx = with_retry(|| async { client.get_transaction(tx_hash).await })
            .await
            .map_err(|_| BlockchainError::TxNotFound(tx_hash.to_string()))?
            .expect("tx not found"); // this should not happen
        let tx_nonce = tx.nonce.as_u64();
        deposited_events.push(Deposited {
            deposit_id: event.deposit_id.try_into().unwrap(),
            sender: event.sender,
            recipient_salt_hash: Bytes32::from_bytes_be(&event.recipient_salt_hash),
            token_index: event.token_index,
            amount: U256::from_bytes_be(&u256_as_bytes_be(event.amount)),
            tx_nonce,
        });
    }
    deposited_events.sort_by_key(|event| event.deposit_id);
    Ok(deposited_events)
}

#[derive(Clone, Debug)]
pub struct DepositLeafInserted {
    pub deposit_index: u32,
    pub deposit_hash: Bytes32,
    pub block_number: u64,
}

pub async fn get_deposit_leaf_inserted_event(
    from_block: u64,
) -> Result<Vec<DepositLeafInserted>, BlockchainError> {
    const PAGE_SIZE: u64 = 500000;

    let latest_block_number = get_client()
        .await?
        .get_block_number()
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to get latest block number".to_string())
        })?
        .as_u64();
    let mut flatten_events = Vec::new();
    let mut from_block = from_block;
    let mut end_block = from_block + PAGE_SIZE - 1;
    while from_block <= latest_block_number {
        if end_block > latest_block_number {
            end_block = latest_block_number;
        }

        let int1 = get_int1_contract().await?;
        let events = with_retry(|| async {
            int1.deposit_leaf_inserted_filter()
                .address(int1.address().into())
                .from_block(from_block)
                .to_block(end_block)
                .query_with_meta()
                .await
        })
        .await
        .map_err(|_| {
            BlockchainError::NetworkError("failed to get deposit leaf inserted event".to_string())
        })?;

        let events: Vec<DepositLeafInserted> = events
            .into_iter()
            .map(|(event, meta)| DepositLeafInserted {
                deposit_index: event.deposit_index,
                deposit_hash: Bytes32::from_bytes_be(&event.deposit_hash),
                block_number: meta.block_number.as_u64(),
            })
            .collect();
        flatten_events.extend_from_slice(&events);

        from_block = end_block + 1;
        end_block = from_block + PAGE_SIZE - 1;
    }

    flatten_events.sort_by_key(|event| event.deposit_index);
    Ok(flatten_events)
}

pub async fn get_latest_deposit_timestamp(sender: Address) -> Result<Option<u64>, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let events = with_retry(|| async {
        int1.deposited_filter()
            .from_block(0)
            .address(int1.address().into())
            .topic2(sender)
            .query_with_meta()
            .await
    })
    .await
    .map_err(|_| BlockchainError::NetworkError("failed to get deposited event".to_string()))?;
    let latest_block_number: Option<u64> = events
        .into_iter()
        .map(|(_, meta)| meta.block_number.as_u64())
        .max();
    let block_timestamp = if let Some(block_number) = latest_block_number {
        let block = get_block(block_number).await?;
        Some(block.unwrap().timestamp.as_u64())
    } else {
        None
    };
    Ok(block_timestamp)
}

pub async fn get_latest_withdrawal_timestamp(
    recipient: Address,
) -> Result<Option<u64>, BlockchainError> {
    let int1 = get_int1_contract().await?;
    let events = with_retry(|| async {
        int1.withdrawn_filter()
            .from_block(0)
            .address(int1.address().into())
            .topic1(recipient)
            .query_with_meta()
            .await
    })
    .await
    .map_err(|_| BlockchainError::NetworkError("failed to get withdrawn event".to_string()))?;
    let latest_block_number: Option<u64> = events
        .into_iter()
        .map(|(_, meta)| meta.block_number.as_u64())
        .max();
    let block_timestamp = if let Some(block_number) = latest_block_number {
        let block = get_block(block_number).await?;
        Some(block.unwrap().timestamp.as_u64())
    } else {
        None
    };
    Ok(block_timestamp)
}

#[cfg(test)]
mod tests {
    use ethers::types::Address;

    use super::*;

    #[tokio::test]
    async fn test_get_deposited_event() {
        let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
            .parse()
            .unwrap();
        let events = get_deposited_event_by_sender(sender).await.unwrap();
        dbg!(events);
    }

    #[tokio::test]
    async fn test_get_deposit_leaf_inserted_event() {
        let events = get_deposit_leaf_inserted_event(0).await.unwrap();
        dbg!(events);
    }
}
