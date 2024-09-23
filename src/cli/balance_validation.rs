use ethers::types::{Address, H256};

use crate::state::state::State;

pub async fn validate_deposit_address(
    satet: &mut State,
    deposit_private_key: H256,
    deposit_address: Address,
) -> anyhow::Result<()> {
    Ok(())
}
