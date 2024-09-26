use ethers::{
    contract::abigen,
    providers::{Http, Provider},
    types::{Address, U256},
};
use log::info;

use super::utils::get_client;

abigen!(
    IERC20,
    r#"[
            function totalSupply() external view returns (uint256)
            function balanceOf(address account) external view returns (uint256)
            function transfer(address recipient, uint256 amount) external returns (bool)
            function allowance(address owner, address spender) external view returns (uint256)
            function approve(address spender, uint256 amount) external returns (bool)
            function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
            event Transfer(address indexed from, address indexed to, uint256 value)
            event Approval(address indexed owner, address indexed spender, uint256 value)
        ]"#,
);

pub async fn get_token_contract() -> anyhow::Result<IERC20<Provider<Http>>> {
    info!("Getting token contract");
    let settings = crate::utils::config::Settings::load()?;
    let client = get_client().await?;
    let token_address: Address = settings.blockchain.token_address.parse()?;
    let contract = IERC20::new(token_address, client);
    Ok(contract)
}

pub async fn get_token_balance(address: Address) -> anyhow::Result<U256> {
    let contract = get_token_contract().await?;
    let balance = contract.balance_of(address).call().await?;
    Ok(balance)
}
