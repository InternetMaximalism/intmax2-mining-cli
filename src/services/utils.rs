use alloy::{
    primitives::{utils::format_units, Address, U256},
    providers::Provider as _,
};

use crate::{
    cli::console::{print_status, print_warning},
    external_api::contracts::utils::NormalProvider,
    utils::{config::Settings, env_config::EnvConfig, time::sleep_for},
};

pub async fn insufficient_balance_instruction(
    provider: &NormalProvider,
    address: Address,
    required_balance: U256,
    name: &str,
) -> anyhow::Result<()> {
    let balance = provider.get_balance(address).await?;
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
        let new_balance = provider.get_balance(address).await?;
        if new_balance > required_balance {
            print_status("Balance updated");
            sleep_for(10);
            break;
        }
        sleep_for(10);
    }
    Ok(())
}

pub async fn await_until_low_gas_price(provider: &NormalProvider) -> anyhow::Result<()> {
    let max_gas_price = EnvConfig::import_from_env()?.max_gas_price;
    let settings = Settings::load()?;
    let high_gas_retry_interval_in_sec = settings.service.high_gas_retry_interval_in_sec;
    let _url = settings.service.repository_url;
    loop {
        let current_gas_price = U256::from(provider.get_gas_price().await?);
        if current_gas_price <= max_gas_price {
            log::info!(
                "Current gas price: {} GWei is lower than max gas price: {} GWei",
                format_units(current_gas_price.clone(), "gwei").unwrap(),
                format_units(max_gas_price.clone(), "gwei").unwrap(),
            );
            break;
        }
        print_warning(format!(
            "Current gas price: {} Gwei > max gas price: {} Gwei. Waiting for gas price to drop...",
            format_units(current_gas_price.clone(), "gwei").unwrap(),
            format_units(max_gas_price.clone(), "gwei").unwrap(),
        ));
        sleep_for(high_gas_retry_interval_in_sec);
    }
    Ok(())
}

pub async fn is_address_used(
    provider: &NormalProvider,
    deposit_address: Address,
) -> anyhow::Result<bool> {
    let account = provider.get_account(deposit_address).await?;
    let nonce = account.nonce;
    let balance = account.balance;
    Ok(nonce > 0 || balance > U256::default())
}

pub fn pretty_format_u256(value: U256) -> String {
    let s = format_units(value, "ether").unwrap();
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

#[cfg(test)]
mod tests {
    use alloy::primitives::utils::parse_ether;

    #[test]
    fn test_pretty_format() {
        let value = parse_ether("1.01000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1.01");

        let value = parse_ether("1.00000000000000000").unwrap();
        let pretty = super::pretty_format_u256(value);
        assert_eq!(pretty, "1");
    }
}
