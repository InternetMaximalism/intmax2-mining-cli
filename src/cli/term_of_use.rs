use std::path::PathBuf;

use dialoguer::Confirm;

use crate::{
    constants::{BRANCH, REPO_NAME, REPO_OWNER, TERM_OF_USE_PATH},
    utils::file::{create_file_with_content, get_data_path},
};

fn agreement_path() -> PathBuf {
    get_data_path().unwrap().join("agreement")
}

pub struct Agreement {}

impl Agreement {
    pub fn save() -> anyhow::Result<()> {
        create_file_with_content(&agreement_path(), &[])
    }

    pub fn is_accepted() -> bool {
        agreement_path().exists()
    }
}

pub fn make_agreement() -> anyhow::Result<()> {
    if Agreement::is_accepted() {
        return Ok(());
    }
    let term_of_use_url = format!(
        "https://github.com/{}/{}/blob/{}/{}",
        REPO_OWNER, REPO_NAME, BRANCH, TERM_OF_USE_PATH
    );
    println!("Please read and accept the terms of use");
    println!("{}", term_of_use_url);
    let accept = Confirm::new()
        .default(false)
        .with_prompt("Do you accept the terms of use?")
        .interact()?;
    if !accept {
        anyhow::bail!("You must accept the terms of use to continue.");
    }
    Agreement::save()?;
    Ok(())
}
