use super::{convert::convert_bytes32_to_b256, error::BlockchainError, utils::NormalProvider};
use alloy::{primitives::Address, sol};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};

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
}
