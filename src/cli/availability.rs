use crate::external_api::intmax::availability::get_availability;

pub async fn check_avaliability() -> anyhow::Result<()> {
    let output = get_availability().await?;
    if !output.is_available {
        anyhow::bail!(output.message);
    }
    Ok(())
}
