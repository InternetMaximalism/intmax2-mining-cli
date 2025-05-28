use crate::external_api::contracts::{
    convert::{convert_address_to_alloy, convert_u256_to_alloy},
    utils::get_provider_with_signer,
};

use super::{
    convert::convert_bytes32_to_b256, error::BlockchainError,
    handlers::send_transaction_with_gas_bump, utils::NormalProvider,
};
use alloy::{
    primitives::{Address, Bytes, B256},
    sol,
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};
use mining_circuit_v1::claim::{claim_circuit::ClaimPublicInputs, mining_claim::MiningClaim};

sol!(
    #[sol(rpc)]
    MinterV1,
    "abi/MinterV1L.json",
);

#[derive(Debug, Clone)]
pub struct MinterContract {
    pub provider: NormalProvider,
    pub address: Address,
}

impl MinterContract {
    pub fn new(provider: NormalProvider, address: Address) -> Self {
        Self { provider, address }
    }

    pub async fn get_short_term_eligible_root(&self) -> Result<Bytes32, BlockchainError> {
        let minter = MinterV1::new(self.address, self.provider.clone());
        let root = minter.shortTermEligibleTreeRoot().call().await?;
        Ok(Bytes32::from_bytes_be(&root.to_vec()))
    }

    pub async fn get_long_term_eligible_root(&self) -> Result<Bytes32, BlockchainError> {
        let minter = MinterV1::new(self.address, self.provider.clone());
        let root = minter.longTermEligibleTreeRoot().call().await?;
        Ok(Bytes32::from_bytes_be(&root.to_vec()))
    }

    pub async fn get_short_term_claim_nullifier_exists(
        &self,
        nullifier: Bytes32,
    ) -> Result<bool, BlockchainError> {
        let minter = MinterV1::new(self.address, self.provider.clone());
        let nullifier = convert_bytes32_to_b256(nullifier);
        let exists = minter.shortTermNullifiers(nullifier).call().await?;
        Ok(exists)
    }

    pub async fn get_long_term_claim_nullifier_exists(
        &self,
        nullifier: Bytes32,
    ) -> Result<bool, BlockchainError> {
        let minter = MinterV1::new(self.address, self.provider.clone());
        let nullifier = convert_bytes32_to_b256(nullifier);
        let exists = minter.longTermNullifiers(nullifier).call().await?;
        Ok(exists)
    }

    pub async fn claim_tokens(
        &self,
        signer_private_key: B256,
        is_short_term: bool,
        claims: &[MiningClaim],
        pis: &ClaimPublicInputs,
        proof: Bytes,
    ) -> Result<(), BlockchainError> {
        let signer = get_provider_with_signer(&self.provider, signer_private_key);
        let contract = MinterV1::new(self.address, signer.clone());
        let claims = claims
            .iter()
            .map(|claim| IMinterV1L::MintClaim {
                recipient: convert_address_to_alloy(claim.recipient),
                nullifier: convert_bytes32_to_b256(claim.nullifier),
                amount: convert_u256_to_alloy(claim.amount),
            })
            .collect::<Vec<_>>();
        let pis = IMinterV1L::ClaimPublicInputs {
            depositTreeRoot: convert_bytes32_to_b256(pis.deposit_tree_root),
            eligibleTreeRoot: convert_bytes32_to_b256(pis.eligible_tree_root),
            lastClaimHash: convert_bytes32_to_b256(pis.last_claim_hash),
        };
        let tx_request = contract
            .claimTokens(is_short_term, claims, pis, proof)
            .into_transaction_request();
        send_transaction_with_gas_bump(signer, tx_request, "claim_tokens").await?;
        Ok(())
    }
}

// pub async fn claim_tokens(
//     claim_key: B256,
//     is_short_term: bool,
//     claims: &[MiningClaim],
//     pis: ClaimPublicInputs,
//     proof: &str,
// ) -> anyhow::Result<()> {
//     info!(
//         "Calling claim_tokens: claims {:?}, pis {:?}, proof {:?}",
//         claims, pis, proof
//     );
//     let mut mint_claims = Vec::<minter_v1::MintClaim>::new();
//     for claim in claims {
//         mint_claims.push(minter_v1::MintClaim {
//             recipient: Address::from_slice(&claim.recipient.to_bytes_be()),
//             nullifier: claim.nullifier.to_bytes_be().try_into().unwrap(),
//             amount: U256::from_big_endian(&claim.amount.to_bytes_be()),
//         });
//     }
//     let pis = minter_v1::ClaimPublicInputs {
//         deposit_tree_root: pis.deposit_tree_root.to_bytes_be().try_into().unwrap(),
//         eligible_tree_root: pis.eligible_tree_root.to_bytes_be().try_into().unwrap(),
//         last_claim_hash: pis.last_claim_hash.to_bytes_be().try_into().unwrap(),
//     };
//     let proof = Bytes::from_str(proof).unwrap();
//     let claim_address = get_wallet(claim_key).await?.address();
//     print_status(format!("Claiming tokens for address: {}", claim_address));

//     await_until_low_gas_price().await?;
//     let minter = get_minter_contract_with_signer(claim_key).await?;
//     let mut tx = minter.claim_tokens(
//         is_short_term,
//         mint_claims.clone(),
//         pis.clone(),
//         proof.clone(),
//     );
//     set_gas_price(&mut tx).await?;
//     info!("Calling claim_tokens: tx {:?}", tx);
//     let _tx_hash = handle_contract_call(tx, claim_address, "claim", "claim").await?;
//     print_log(format!("Successfully claimed"));
//     Ok(())
// }
