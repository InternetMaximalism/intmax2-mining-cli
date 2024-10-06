use anyhow::bail;

use crate::external_api::intmax::{availability::get_availability, error::IntmaxError};

use super::console::print_error;

pub async fn check_avaliability() -> anyhow::Result<()> {
    match get_availability().await {
        Ok(output) => {
            if !output.is_available {
                print_error(&output.message);
                let do_update = dialoguer::Confirm::new()
                    .with_prompt("Do you want to update the CLI?")
                    .default(true)
                    .interact()?;
                if do_update {
                    crate::utils::update::update()?;
                } else {
                    bail!("CLI is not available");
                }
            }
            Ok(())
        }
        Err(e) => match e {
            IntmaxError::ServerError(intmax_error_response) => {
                if intmax_error_response.code == "FORBIDDEN" {
                    print_error(&intmax_error_response.message);
                    bail!("CLI is not available");
                } else {
                    bail!(intmax_error_response.message);
                }
            }
            _ => Err(e.into()),
        },
    }
}
