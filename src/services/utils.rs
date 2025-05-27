use alloy::primitives::{Address, U256};

use crate::{
    cli::console::{print_status, print_warning},
    utils::{config::Settings, env_config::EnvConfig, time::sleep_for},
};

pub async fn insufficient_balance_instruction(
    address: Address,
    required_balance: U256,
    name: &str,
) -> anyhow::Result<()> {
    let balance = get_balance(address).await?;
    if required_balance <= balance {
        return Ok(());
    }
    print_warning(format!(
        r"{} address {:?} has insufficient balance {} ETH < {} ETH. Waiting for your deposit...",
        name,
        address,
        pretty_format_u256(balance),
        pretty_format_u256(required_balance)
    ));
    loop {
        let new_balance = get_balance(address).await?;
        if new_balance > required_balance {
            print_status("Balance updated");
            sleep_for(10);
            break;
        }
        sleep_for(10);
    }
    Ok(())
}

pub async fn await_until_low_gas_price() -> anyhow::Result<()> {
    let max_gas_price = EnvConfig::import_from_env()?.max_gas_price;
    let settings = Settings::load()?;
    let high_gas_retry_inverval_in_sec = settings.service.high_gas_retry_inverval_in_sec;
    let _url = settings.service.repository_url;
    loop {
        let current_gas_price = get_gas_price().await?;
        if current_gas_price <= max_gas_price {
            log::info!(
                "Current gas price: {} GWei is lower than max gas price: {} GWei",
                ethers::utils::format_units(current_gas_price.clone(), "gwei").unwrap(),
                ethers::utils::format_units(max_gas_price.clone(), "gwei").unwrap(),
            );
            break;
        }
        print_warning(format!(
            "Current gas price: {} Gwei > max gas price: {} Gwei. Waiting for gas price to drop...",
            ethers::utils::format_units(current_gas_price.clone(), "gwei").unwrap(),
            ethers::utils::format_units(max_gas_price.clone(), "gwei").unwrap(),
        ));
        sleep_for(high_gas_retry_inverval_in_sec);
    }
    Ok(())
}

pub async fn is_address_used(deposit_address: Address) -> bool {
    get_account_nonce(deposit_address).await.unwrap() > 0
        || get_balance(deposit_address).await.unwrap() > 0.into()
}

pub fn pretty_format_u256(value: U256) -> String {
    let s = ethers::utils::format_units(value, "ether").unwrap();
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pretty_format() {
        let value = ethers::utils::parse_ether("1.01000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1.01");

        let value = ethers::utils::parse_ether("1.00000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1");
    }
}
