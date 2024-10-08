use ethers::{
    providers::{Http, PendingTransaction},
    types::{Address, Bytes, H256, U256},
};
use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait;
use log::info;
use mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    external_api::contracts::int1::{get_int1_contract_with_signer, int_1},
    utils::{
        config::Settings,
        network::{get_network, Network},
        retry::with_retry,
    },
};

use super::error::{IntmaxError, IntmaxErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitWithdrawalInput {
    pub public_inputs: SimpleWithdrawalPublicInputs,
    pub proof: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitWithdrawalSuccess {
    pub transaction_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum SumbitWithdrawalResponse {
    Sucess(SubmitWithdrawalSuccess),
    Error(IntmaxErrorResponse),
}

pub async fn submit_withdrawal(
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> Result<H256, IntmaxError> {
    info!("submit_withdrawal with args {:?} proof {}", pis, proof);
    let settings = Settings::load().unwrap();
    let tx_hash = if get_network() == Network::Localnet {
        let tx_hash = localnet_withdrawal(pis, proof).await.unwrap();
        tx_hash
    } else {
        let input = SubmitWithdrawalInput {
            public_inputs: pis,
            proof: "0x".to_string() + proof, // add 0x prefix
        };
        info!(
            "Submitting withdrawal to {}, body: {:?}",
            settings.api.withdrawal_server_url, input
        );
        let response = with_retry(|| async {
            reqwest::Client::new()
                .post(settings.api.withdrawal_server_url.clone())
                .json(&input)
                .send()
                .await
        })
        .await
        .map_err(|_| {
            IntmaxError::NetworkError("failed to request withdrawal server".to_string())
        })?;
        let response: SumbitWithdrawalResponse = response.json().await.map_err(|e| {
            IntmaxError::SerializeError(format!("failed to parse response: {}", e.to_string()))
        })?;
        match response {
            SumbitWithdrawalResponse::Sucess(success) => H256::from_str(&success.transaction_hash)
                .map_err(|_| {
                    IntmaxError::SerializeError("failed to parse transaction hash".to_string())
                })?,
            SumbitWithdrawalResponse::Error(error) => Err(IntmaxError::ServerError(error))?,
        }
    };
    Ok(tx_hash)
}

async fn localnet_withdrawal(
    pis: SimpleWithdrawalPublicInputs,
    proof: &str,
) -> anyhow::Result<H256> {
    // Hardhat's default private key for withdrawal
    let local_private_key =
        H256::from_str("0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a")
            .unwrap();
    let int1 = get_int1_contract_with_signer(local_private_key).await?;
    let public_inputs = int_1::WithdrawalPublicInputs {
        deposit_root: pis.deposit_root.to_bytes_be().try_into().unwrap(),
        nullifier: pis.nullifier.to_bytes_be().try_into().unwrap(),
        recipient: Address::from_slice(&pis.recipient.to_bytes_be()),
        token_index: pis.token_index,
        amount: U256::from_big_endian(&pis.amount.to_bytes_be()),
    };
    let proof = Bytes::from_str(proof)?;
    let tx = int1.withdraw(public_inputs, proof);
    let pending_tx: PendingTransaction<Http> = match tx.send().await {
        Ok(tx) => {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            tx
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
        }
    };
    Ok(pending_tx.tx_hash())
}

#[cfg(test)]
mod tests {
    use intmax2_zkp::ethereum_types::{address::Address, bytes32::Bytes32, u256::U256};

    #[tokio::test]
    #[ignore]
    async fn test_submit_withdrawal() {
        let pis =
            mining_circuit_v1::withdrawal::simple_withraw_circuit::SimpleWithdrawalPublicInputs {
                deposit_root: Bytes32::default(),
                nullifier: Bytes32::default(),
                recipient: Address::default(),
                token_index: 0,
                amount: U256::default(),
            };
        let proof = "0x12345678";
        let tx_hash = super::submit_withdrawal(pis, proof).await.unwrap();
        println!("tx_hash: {:?}", tx_hash);
    }
}
