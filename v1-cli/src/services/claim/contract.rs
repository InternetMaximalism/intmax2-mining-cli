use std::str::FromStr;

use ethers::{
    signers::Signer as _,
    types::{Address, Bytes, H256, U256},
};
use intmax2_zkp::ethereum_types::u32limb_trait::U32LimbTrait;
use mining_circuit_v1::claim::{claim_circuit::ClaimPublicInputs, mining_claim::MiningClaim};

use crate::{
    cli::console::print_status,
    external_api::contracts::{
        minter::{get_minter_contract_with_signer, minter_v1},
        utils::get_wallet,
    },
    services::contracts::handle_contract_call,
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
    print_status(format!("Claiming tokens for address: {}", claim_address));
    let minter = get_minter_contract_with_signer(claim_key).await?;
    let tx = minter.claim_tokens(mint_claims.clone(), pis.clone(), proof.clone());
    handle_contract_call(tx, claim_address, "claim", "claim").await?;
    Ok(())
}
