use super::{error::BlockchainError, utils::NormalProvider};
use alloy::{
    primitives::{Address, U256},
    sol,
};

sol!(
    #[sol(rpc)]
    ERC20,
    "abi/ERC20.json",
);
pub struct TokenContract {
    pub provider: NormalProvider,
    pub address: Address,
}

impl TokenContract {
    pub fn new(provider: NormalProvider, address: Address) -> Self {
        Self { provider, address }
    }

    pub async fn get_token_balance(&self, address: Address) -> Result<U256, BlockchainError> {
        let contract = ERC20::new(self.address, self.provider.clone());
        let balance = contract.balanceOf(address).call().await?;
        Ok(balance)
    }
}
