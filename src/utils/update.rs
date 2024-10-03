use self_update::cargo_crate_version;

use crate::constants::{REPO_NAME, REPO_OWNER};

pub fn update() -> anyhow::Result<()> {
    let _status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("mining-cli")
        .show_output(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::update;

    #[test]
    fn test_update() -> anyhow::Result<()> {
        update()?;
        Ok(())
    }
}
