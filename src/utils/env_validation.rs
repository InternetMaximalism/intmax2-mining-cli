use super::{config::Settings, env_config::EnvConfig};
use crate::state::keys::Keys;
use ethers::{
    providers::Middleware as _,
    types::{H256, U256},
};

pub async fn validate_env_config(env: &EnvConfig, keys: &Keys) -> anyhow::Result<()> {
    validate_rpc_url(&env.rpc_url).await?;
    validate_mining_unit(env.mining_unit)?;
    validate_mining_times(env.mining_times)?;
    validate_keys(&keys)?;
    Ok(())
}

fn validate_mining_unit(mining_unit: U256) -> anyhow::Result<()> {
    let one_tenth: U256 = ethers::utils::parse_ether("0.1").unwrap().into();
    let one: U256 = ethers::utils::parse_ether("1").unwrap().into();
    if mining_unit != one_tenth && mining_unit != one {
        anyhow::bail!("MINING_UNIT environment variable must be either '1' or '0.1'");
    }
    Ok(())
}

fn validate_mining_times(mining_times: u64) -> anyhow::Result<()> {
    if mining_times != 10 && mining_times != 100 {
        anyhow::bail!("MINING_TIMES environment variable must be either '10' or '100'");
    }
    Ok(())
}

fn validate_keys(keys: &Keys) -> anyhow::Result<()> {
    if keys.deposit_private_keys.is_empty() {
        anyhow::bail!("Deposit private keys are empty");
    }
    if keys.deposit_private_keys.contains(&H256::zero()) {
        anyhow::bail!("Deposit private keys contain zero");
    }
    if keys.withdrawal_private_key == H256::zero() {
        anyhow::bail!("Withdrawal private key is zero");
    }
    if keys.deposit_addresses.contains(&keys.withdrawal_address) {
        anyhow::bail!("Deposit addresses contain withdrawal address");
    }
    Ok(())
}

async fn validate_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let client = ethers::providers::Provider::<ethers::providers::Http>::try_from(rpc_url)?;
    let chain_id = client.get_chainid().await?;
    let setting = Settings::load()?;
    if chain_id != setting.blockchain.chain_id.into() {
        return Err(anyhow::anyhow!(
            "RPC URL chain id {} does not match the expected chain id {}",
            chain_id,
            setting.blockchain.chain_id
        ));
    }
    Ok(())
}
