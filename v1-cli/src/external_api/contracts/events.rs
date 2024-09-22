use ethers::{providers::Middleware as _, types::Address};
use intmax2_zkp::{
    common::deposit::Deposit,
    ethereum_types::{bytes32::Bytes32, u256::U256, u32limb_trait::U32LimbTrait},
};

use super::{
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

pub async fn get_deposited_event_by_sender(sender: Address) -> anyhow::Result<Vec<Deposited>> {
    let int1 = get_int1_contract().await?;
    let events = int1
        .deposited_filter()
        .from_block(0)
        .topic2(sender)
        .query_with_meta()
        .await?;
    let client = get_client().await?;

    let mut deposited_events = Vec::new();
    for (event, meta) in events {
        // get tx_nonce  because it is needed for getting deposit salt
        let tx_hash = meta.transaction_hash;
        let tx = client
            .get_transaction(tx_hash)
            .await?
            .expect("tx not found");
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
) -> anyhow::Result<Vec<DepositLeafInserted>> {
    let int1 = get_int1_contract().await?;
    let events = int1
        .deposit_leaf_inserted_filter()
        .from_block(from_block)
        .query_with_meta()
        .await?;
    let mut events: Vec<DepositLeafInserted> = events
        .into_iter()
        .map(|(event, meta)| DepositLeafInserted {
            deposit_index: event.deposit_index,
            deposit_hash: Bytes32::from_bytes_be(&event.deposit_hash),
            block_number: meta.block_number.as_u64(),
        })
        .collect();
    events.sort_by_key(|event| event.deposit_index);
    Ok(events)
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
