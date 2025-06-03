use alloy::{
    primitives::{utils::parse_ether, U256},
    providers::{Provider, ProviderBuilder},
};
use reqwest::Url;

use super::{
    config::Settings,
    env_config::EnvConfig,
    network::{get_network, Network},
};

pub fn get_allowed_mining_times() -> Vec<u64> {
    let network = get_network();
    if network == Network::BaseSepolia {
        vec![1, 5, 10]
    } else if network == Network::Base {
        vec![1, 10]
    } else {
        vec![10]
    }
}

pub async fn validate_env_config(env: &EnvConfig) -> anyhow::Result<()> {
    validate_rpc_url(&env.rpc_url).await?;
    validate_mining_unit(env.mining_unit)?;
    validate_mining_times(env.mining_times)?;
    Ok(())
}

fn validate_mining_unit(mining_unit: U256) -> anyhow::Result<()> {
    let one_tenth: U256 = parse_ether("0.1").unwrap();
    let one: U256 = parse_ether("1").unwrap();
    if mining_unit != one_tenth && mining_unit != one {
        anyhow::bail!("MINING_UNIT environment variable must be either '1' or '0.1'");
    }
    Ok(())
}

fn validate_mining_times(mining_times: u64) -> anyhow::Result<()> {
    let allowed_mining_times = get_allowed_mining_times();
    if !allowed_mining_times.contains(&mining_times) {
        anyhow::bail!(
            "MINING_TIMES environment variable must be one of {:?}",
            allowed_mining_times
        );
    }
    Ok(())
}

pub async fn validate_rpc_url(rpc_url: &str) -> anyhow::Result<()> {
    let rpc_url: Url = rpc_url
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse RPC URL {}: {}", rpc_url, e))?;

    let client = ProviderBuilder::new().connect_http(rpc_url);
    let chain_id = client.get_chain_id().await?;
    let setting = Settings::load()?;
    if chain_id != setting.blockchain.chain_id {
        return Err(anyhow::anyhow!(
            "RPC URL chain id {} does not match the expected chain id {}",
            chain_id,
            setting.blockchain.chain_id
        ));
    }
    Ok(())
}
