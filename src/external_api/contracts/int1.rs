use super::{convert::convert_bytes32_to_b256, error::BlockchainError, utils::NormalProvider};
use alloy::{
    primitives::{Address, U256},
    sol,
};
use intmax2_zkp::ethereum_types::{bytes32::Bytes32, u32limb_trait::U32LimbTrait};

sol!(
    #[sol(rpc)]
    Int1,
    "abi/Int1L.json",
);

#[derive(Debug, Clone)]
pub struct Int1Contract {
    pub provider: NormalProvider,
    pub address: Address,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DepositData {
    pub deposit_hash: Bytes32,
    pub sender: Address,
    pub is_rejected: bool,
}

impl Int1Contract {
    pub fn new(provider: NormalProvider, address: Address) -> Self {
        Self { provider, address }
    }

    pub async fn get_deposit_root(&self) -> Result<Bytes32, BlockchainError> {
        let int1 = Int1::new(self.address, self.provider.clone());
        let root = int1.getDepositRoot().call().await?;
        Ok(Bytes32::from_bytes_be(&root.to_vec()))
    }

    pub async fn get_deposit_root_exits(&self, root: Bytes32) -> Result<bool, BlockchainError> {
        let int1 = Int1::new(self.address, self.provider.clone());
        let root = convert_bytes32_to_b256(root);
        let block_number = int1.depositRoots(root).call().await?;
        Ok(!block_number.is_zero())
    }

    pub async fn get_deposit_data(&self, deposit_id: u64) -> Result<DepositData, BlockchainError> {
        let int1 = Int1::new(self.address, self.provider.clone());
        let data = int1.getDepositData(U256::from(deposit_id)).call().await?;
        let data = DepositData {
            deposit_hash: Bytes32::from_bytes_be(&data.depositHash.to_vec()),
            sender: data.sender,
            is_rejected: data.isRejected,
        };
        Ok(data)
    }

    pub async fn get_withdrawal_nullifier_exists(
        &self,
        nullifier: Bytes32,
    ) -> Result<bool, BlockchainError> {
        let int1 = Int1::new(self.address, self.provider.clone());
        let nullifier = convert_bytes32_to_b256(nullifier);
        let block_number = int1.nullifiers(nullifier).call().await?;
        Ok(!block_number.is_zero())
    }

    pub async fn get_last_processed_deposit_id(&self) -> Result<u64, BlockchainError> {
        let int1 = Int1::new(self.address, self.provider.clone());
        let id = int1.getLastProcessedDepositId().call().await?;
        Ok(id.to())
    }
}
