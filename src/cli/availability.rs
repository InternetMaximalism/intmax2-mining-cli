use crate::external_api::intmax::availability::get_availability;

pub async fn check_avaliability() -> anyhow::Result<()> {
    let output = get_availability().await?;
    println!("{}", output.message);
    if !output.is_available {
        return Err(anyhow::anyhow!("Availability check failed"));
    }
    Ok(())
}
