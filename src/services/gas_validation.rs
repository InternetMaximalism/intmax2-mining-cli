use crate::{
    cli::console::print_warning,
    external_api::contracts::utils::get_gas_price,
    utils::{config::Settings, env_config::EnvConfig},
};

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
            "Current gas price: {} GWei is higher than max gas price: {} GWei. Please see README",
            ethers::utils::format_units(current_gas_price.clone(), "gwei").unwrap(),
            ethers::utils::format_units(max_gas_price.clone(), "gwei").unwrap(),
        ));
        tokio::time::sleep(std::time::Duration::from_secs(
            high_gas_retry_inverval_in_sec,
        ))
        .await;
    }
    Ok(())
}
