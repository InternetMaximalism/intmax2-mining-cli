use alloy::primitives::Address;
use intmax2_zkp::{
    common::deposit::Deposit,
    ethereum_types::{bytes32::Bytes32, u256::U256},
};

#[derive(Clone, Debug)]
pub struct Deposited {
    pub deposit_id: u64,
    pub sender: Address,
    pub recipient_salt_hash: Bytes32,
    pub token_index: u32,
    pub amount: U256,
    pub tx_nonce: u64,
    pub timestamp: u64,
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

#[derive(Clone, Debug)]
pub struct DepositLeafInserted {
    pub deposit_index: u32,
    pub deposit_hash: Bytes32,
    pub block_number: u64,
}

// pub async fn get_latest_deposit_timestamp(sender: Address) -> Result<Option<u64>, BlockchainError> {
//     log::info!("get_latest_deposit_timestamp");
//     let int1 = get_int1_contract().await?;

//     let mut to_block = get_latest_block_number().await?;
//     let int1_deployed_block = crate::utils::config::Settings::load()
//         .unwrap()
//         .blockchain
//         .int1_deployed_block;
//     loop {
//         let events = with_retry(|| async {
//             int1.deposited_filter()
//                 .from_block(to_block.saturating_sub(EVENT_BLOCK_RANGE))
//                 .to_block(to_block)
//                 .address(int1.address().into())
//                 .topic2(sender)
//                 .query_with_meta()
//                 .await
//         })
//         .await
//         .map_err(|_| BlockchainError::NetworkError("failed to get deposited event".to_string()))?;
//         let max_block_number: Option<u64> = events
//             .into_iter()
//             .map(|(_, meta)| meta.block_number.as_u64())
//             .max();
//         if let Some(max_block_number) = max_block_number {
//             let block = get_block(max_block_number).await?;
//             return Ok(Some(block.unwrap().timestamp.as_u64()));
//         }
//         to_block = to_block.saturating_sub(EVENT_BLOCK_RANGE);
//         if to_block < int1_deployed_block {
//             return Ok(None);
//         }
//     }
// }

// pub async fn get_latest_withdrawal_timestamp(
//     recipient: Address,
// ) -> Result<Option<u64>, BlockchainError> {
//     log::info!("get_latest_withdrawal_timestamp");
//     let int1 = get_int1_contract().await?;

//     let mut to_block = get_latest_block_number().await?;
//     let int1_deployed_block = crate::utils::config::Settings::load()
//         .unwrap()
//         .blockchain
//         .int1_deployed_block;
//     loop {
//         let events = with_retry(|| async {
//             int1.withdrawn_filter()
//                 .from_block(to_block.saturating_sub(EVENT_BLOCK_RANGE))
//                 .to_block(to_block)
//                 .address(int1.address().into())
//                 .topic1(recipient)
//                 .query_with_meta()
//                 .await
//         })
//         .await
//         .map_err(|_| BlockchainError::NetworkError("failed to get withdrawn event".to_string()))?;
//         let max_block_number: Option<u64> = events
//             .into_iter()
//             .map(|(_, meta)| meta.block_number.as_u64())
//             .max();
//         if let Some(max_block_number) = max_block_number {
//             let block = get_block(max_block_number).await?;
//             return Ok(Some(block.unwrap().timestamp.as_u64()));
//         }
//         to_block = to_block.saturating_sub(EVENT_BLOCK_RANGE);
//         if to_block < int1_deployed_block {
//             return Ok(None);
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use ethers::types::Address;

//     use super::*;

//     #[tokio::test]
//     async fn test_get_deposited_event() {
//         let sender: Address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
//             .parse()
//             .unwrap();
//         let events = get_deposited_event_by_sender(sender).await.unwrap();
//         dbg!(events);
//     }

//     #[tokio::test]
//     async fn test_get_deposit_leaf_inserted_event() {
//         let events = get_deposit_leaf_inserted_event(0).await.unwrap();
//         dbg!(events);
//     }
// }
