use anyhow::ensure;

use crate::external_api::intmax::availability::get_availability;

pub async fn check_avaliability() -> anyhow::Result<()> {
    let output = get_availability().await?;
    println!("{}", output.message);
    ensure!(output.is_available, "Service is not available");
    Ok(())
}
