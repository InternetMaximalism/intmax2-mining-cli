use std::fs;

use anyhow::Context;
use chrono::NaiveDate;
use log::info;
use regex::Regex;
use reqwest::{self, Client};
use serde_json::Value;

use crate::{
    constants::{CONFIG_BRANCH, CONFIG_PATH, REPO_NAME, REPO_OWNER},
    utils::{
        bin_parser::{BinDepositTree, BinEligibleTree},
        config::Settings,
        file::{create_data_dir, get_data_path},
        retry::with_retry,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to fetch data from GitHub")]
    NetworkError,
    #[error("Failed to deserialize data {}", _0)]
    DeserializeError(String),
}

#[derive(Debug)]
pub struct BinTrees {
    pub bin_deposit_tree: Option<BinDepositTree>,
    pub bin_short_term_eligible_tree: Option<BinEligibleTree>,
    pub bin_long_term_eligible_tree: Option<BinEligibleTree>,
    pub latest_update: NaiveDate,
}

pub async fn fetch_latest_tree_from_github(last_update: NaiveDate) -> Result<BinTrees, Error> {
    info!(
        "Fetching latest deposit and eligible trees from GitHub, last update: {}",
        last_update
    );
    let settings = Settings::load().unwrap();
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/contents/{}?ref={}",
        settings.api.tree_data_repository,
        settings.api.tree_data_directory,
        settings.api.tree_data_branch
    );
    let file_list = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-File-Reader")
        .send()
        .await
        .map_err(|_| Error::NetworkError)?
        .json::<Vec<Value>>()
        .await
        .map_err(|e| {
            Error::DeserializeError(format!("failed to parse fetched files as array: {}", e))
        })?;

    let deposit_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}-depositTree\.txt$").unwrap();
    let short_term_eligible_pattern =
        Regex::new(r"^\d{4}-\d{2}-\d{2}-eligibleTree-shortTerm\.txt$").unwrap();
    let long_term_eligible_pattern =
        Regex::new(r"^\d{4}-\d{2}-\d{2}-eligibleTree-longTerm\.txt$").unwrap();

    let (latest_deposit_date, latest_deposit_file) =
        filter_file(&deposit_pattern, &file_list, last_update);
    let (latest_short_term_eligible_date, latest_short_term_eligible_file) =
        filter_file(&short_term_eligible_pattern, &file_list, last_update);
    let (latest_long_term_eligible_date, latest_long_term_eligible_file) =
        filter_file(&long_term_eligible_pattern, &file_list, last_update);

    let bin_deposit_tree = if let Some(latest_deposit_file) = latest_deposit_file {
        let content = fetch_content(&client, &latest_deposit_file).await?;
        let bin_deposit_tree: BinDepositTree = bincode::deserialize(&content).map_err(|e| {
            Error::DeserializeError(format!("failed to deserialize deposit tree: {}", e))
        })?;
        Some(bin_deposit_tree)
    } else {
        None
    };

    let bin_short_term_eligible_tree = if let Some(latest_short_term_eligible_file) =
        latest_short_term_eligible_file
    {
        let content = fetch_content(&client, &latest_short_term_eligible_file).await?;
        let bin_eligible_tree: BinEligibleTree = bincode::deserialize(&content).map_err(|e| {
            Error::DeserializeError(format!(
                "failed to deserialize short term eligible tree: {}",
                e
            ))
        })?;
        Some(bin_eligible_tree)
    } else {
        None
    };

    let bin_long_term_eligible_tree = if let Some(latest_long_term_eligible_file) =
        latest_long_term_eligible_file
    {
        let content = fetch_content(&client, &latest_long_term_eligible_file).await?;
        let bin_eligible_tree: BinEligibleTree = bincode::deserialize(&content).map_err(|e| {
            Error::DeserializeError(format!(
                "failed to deserialize short term eligible tree: {}",
                e
            ))
        })?;
        Some(bin_eligible_tree)
    } else {
        None
    };
    let latest_update = vec![
        latest_deposit_date,
        latest_short_term_eligible_date,
        latest_long_term_eligible_date,
    ]
    .into_iter()
    .max()
    .unwrap(); // iter is never empty

    Ok(BinTrees {
        bin_deposit_tree,
        bin_short_term_eligible_tree,
        bin_long_term_eligible_tree,
        latest_update,
    })
}

fn filter_file(
    pattern: &Regex,
    file_list: &[Value],
    prev_date: NaiveDate,
) -> (NaiveDate, Option<Value>) {
    let mut latest_file: Option<Value> = None;
    let mut latest_date = prev_date;
    for file in file_list {
        if let Some(name) = file["name"].as_str() {
            if name.len() < 10 {
                // ignore files without date
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&name[0..10], "%Y-%m-%d") {
                if pattern.is_match(name) && date > latest_date {
                    latest_date = date;
                    latest_file = Some(file.clone());
                }
            }
        }
    }
    if latest_file.is_none() || latest_date <= prev_date {
        return (prev_date, None); // no new file
    }
    return (latest_date, latest_file);
}

async fn fetch_content(client: &reqwest::Client, file: &Value) -> Result<Vec<u8>, Error> {
    let download_url = file["download_url"]
        .as_str()
        .ok_or(Error::DeserializeError(
            "no download_url filed in github files".to_string(),
        ))?;
    let content = client
        .get(download_url)
        .header("User-Agent", "Rust-GitHub-File-Reader")
        .send()
        .await
        .map_err(|_| Error::NetworkError)?
        .bytes()
        .await
        .map_err(|_| Error::DeserializeError("failed to deserialize files as bytes".to_string()))?;
    Ok(content.into())
}

pub async fn fetch_config_file_from_github() -> anyhow::Result<()> {
    create_data_dir()?;

    let client = Client::new();
    let files_to_download = vec![
        "config.holesky.toml",
        "config.mainnet.toml",
        "config.base-sepolia.toml",
    ];

    for file_name in files_to_download {
        let file_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
            REPO_OWNER, REPO_NAME, CONFIG_BRANCH, CONFIG_PATH, file_name
        );

        let content = with_retry(|| async { client.get(&file_url).send().await })
            .await
            .context(format!(
                "Failed to fetch config file from GitHub: {}",
                file_name
            ))?;
        let content = content
            .bytes()
            .await
            .context("Failed to read response body")?;
        let file_path = get_data_path()?.join(file_name);
        fs::write(&file_path, content).context(format!("Failed to write file: {}", file_name))?;
        info!("Downloaded: {}", file_path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_latest_files() {
        let last_update = NaiveDate::parse_from_str("2023-01-01", "%Y-%m-%d").unwrap();
        let _result = fetch_latest_tree_from_github(last_update).await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_latest_files_no_new_files() {
        let last_update = NaiveDate::parse_from_str("2999-12-31", "%Y-%m-%d").unwrap();
        let _result = fetch_latest_tree_from_github(last_update).await.unwrap();
    }

    #[tokio::test]
    async fn test_fetch_config_file_from_github() {
        fetch_config_file_from_github().await.unwrap();
    }
}
