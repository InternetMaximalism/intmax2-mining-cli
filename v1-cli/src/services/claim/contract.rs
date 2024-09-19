use std::str::FromStr;

use anyhow::ensure;
use ethers::{
    providers::{Http, PendingTransaction},
    signers::Signer as _,
    types::{Address, Bytes, H256, U256},
};
use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait;
use mining_circuit::claim::{claim_circuit::ClaimPublicInputs, mining_claim::MiningClaim};

use crate::{
    cli::console::{insuffient_balance_instruction, print_status},
    external_api::contracts::{
        minter::{get_minter_contract_with_signer, minter_v1},
        utils::get_wallet,
    },
};

pub async fn claim_tokens(
    claim_key: H256,
    claims: &[MiningClaim],
    pis: ClaimPublicInputs,
    proof: &str,
) -> anyhow::Result<()> {
    let mut mint_claims = Vec::<minter_v1::MintClaim>::new();
    for claim in claims {
        mint_claims.push(minter_v1::MintClaim {
            recipient: Address::from_slice(&claim.recipient.to_bytes_be()),
            nullifier: claim.nullifier.to_bytes_be().try_into().unwrap(),
            amount: U256::from_big_endian(&claim.amount.to_bytes_be()),
        });
    }
    let pis = minter_v1::ClaimPublicInputs {
        deposit_tree_root: pis.deposit_tree_root.to_bytes_be().try_into().unwrap(),
        eligible_tree_root: pis.eligible_tree_root.to_bytes_be().try_into().unwrap(),
        last_claim_hash: pis.last_claim_hash.to_bytes_be().try_into().unwrap(),
    };
    let proof = Bytes::from_str(proof).unwrap();

    let claim_address = get_wallet(claim_key).await?.address();
    print_status(&format!("Claiming tokens for address: {}", claim_address));

    loop {
        let minter = get_minter_contract_with_signer(claim_key);
        let tx = minter
            .await?
            .claim_tokens(mint_claims.clone(), pis.clone(), proof.clone());
        let result = tx.send().await;
        match result {
            Ok(tx) => {
                let pending_tx: PendingTransaction<Http> = tx;
                print_status(&format!("Claim tx hash: {:?}", pending_tx.tx_hash()));
                let tx_receipt = pending_tx.await?.unwrap();
                ensure!(tx_receipt.status.unwrap() == 1.into(), "Claim tx failed");
                return Ok(());
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("-32000") {
                    insuffient_balance_instruction(claim_address, "claim").await?;
                    print_status("Retrying claim transaction...");
                } else {
                    return Err(anyhow::anyhow!("Error sending transaction: {:?}", e));
                }
            }
        }
    }
}
