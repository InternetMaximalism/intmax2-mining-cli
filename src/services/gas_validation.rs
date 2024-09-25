use crate::{
    cli::{console::print_warning, load_env::load_max_gas_price},
    external_api::contracts::utils::get_gas_price,
    utils::config::Settings,
};

pub async fn await_until_low_gas_price() -> anyhow::Result<()> {
    let max_gas_price = load_max_gas_price()?;
    let settings = Settings::new()?;
    let high_gas_retry_inverval_in_sec = settings.service.high_gas_retry_inverval_in_sec;
    let url = settings.service.repository_url;
    loop {
        let current_gas_price = get_gas_price().await?;
        if current_gas_price <= max_gas_price {
            log::info!(
                "Current gas price: {:?} is lower than max gas price: {:?}",
                current_gas_price,
                max_gas_price
            );
            break;
        }
        print_warning(format!(
            "Current gas price: {:?} is higher than max gas price: {:?}. Please see README at {}",
            current_gas_price, max_gas_price, url
        ));
        tokio::time::sleep(std::time::Duration::from_secs(
            high_gas_retry_inverval_in_sec,
        ))
        .await;
    }
    Ok(())
}
