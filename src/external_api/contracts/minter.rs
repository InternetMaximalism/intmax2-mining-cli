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
        Ok(Bytes32::from_bytes_be(root.as_ref()))
    }

    pub async fn get_long_term_eligible_root(&self) -> Result<Bytes32, BlockchainError> {
        let minter = MinterV1::new(self.address, self.provider.clone());
        let root = minter.longTermEligibleTreeRoot().call().await?;
        Ok(Bytes32::from_bytes_be(root.as_ref()))
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
        send_transaction_with_gas_bump(
            &self.provider,
            signer,
            tx_request,
            "claim_tokens",
            "claim address",
        )
        .await?;
        Ok(())
    }
}
