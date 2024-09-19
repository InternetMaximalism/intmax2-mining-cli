use anyhow::Context;
use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use reqwest;
use serde_json::Value;

use crate::{
    config::Settings,
    utils::bin_parser::{BinDepositTree, BinEligibleTree},
};

pub async fn fetch_latest_tree_from_github(
    last_update: NaiveDateTime,
) -> anyhow::Result<Option<(BinDepositTree, BinEligibleTree, NaiveDateTime)>> {
    let settings = Settings::new()?;
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/contents/data",
        settings.api.tree_data_repository
    );
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-File-Reader")
        .send()
        .await?
        .json::<Vec<Value>>()
        .await
        .context("Failed to fetch data from GitHub")?;

    let deposit_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}-depositTree\.txt$").unwrap();
    let eligible_pattern = Regex::new(r"^\d{4}-\d{2}-\d{2}-eligibleTree\.txt$").unwrap();
    let mut latest_deposit_file: Option<&Value> = None;
    let mut latest_eligible_file: Option<&Value> = None;
    let mut latest_deposit_date = last_update.date();
    let mut latest_eligible_date = last_update.date();

    for file in response.iter() {
        if let Some(name) = file["name"].as_str() {
            if name.len() < 10 {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&name[0..10], "%Y-%m-%d") {
                if deposit_pattern.is_match(name) && date > latest_deposit_date {
                    latest_deposit_date = date;
                    latest_deposit_file = Some(file);
                } else if eligible_pattern.is_match(name) && date > latest_eligible_date {
                    latest_eligible_date = date;
                    latest_eligible_file = Some(file);
                }
            }
        }
    }

    if latest_deposit_file.is_none()
        || latest_eligible_file.is_none()
        || (latest_deposit_date <= last_update.date() && latest_eligible_date <= last_update.date())
    {
        return Ok(None);
    }

    let deposit_file = latest_deposit_file.unwrap();
    let content = fetch_content(&client, deposit_file).await?;
    let bin_deposit_tree: BinDepositTree = bincode::deserialize(&content)?;

    let eligible_file = latest_eligible_file.unwrap();
    let content = fetch_content(&client, eligible_file).await?;
    let bin_eligible_tree: BinEligibleTree = bincode::deserialize(&content)?;

    let latest_update = latest_deposit_date
        .max(latest_eligible_date)
        .and_hms_opt(0, 0, 0)
        .unwrap();

    Ok(Some((bin_deposit_tree, bin_eligible_tree, latest_update)))
}

async fn fetch_content(client: &reqwest::Client, file: &Value) -> anyhow::Result<Vec<u8>> {
    let download_url = file["download_url"]
        .as_str()
        .ok_or(anyhow::anyhow!("No download URL"))?;
    let content = client
        .get(download_url)
        .header("User-Agent", "Rust-GitHub-File-Reader")
        .send()
        .await?
        .bytes()
        .await?;
    Ok(content.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[tokio::test]
    async fn test_fetch_latest_files() {
        let last_update =
            NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let result = fetch_latest_tree_from_github(last_update).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_fetch_latest_files_no_new_files() {
        let last_update =
            NaiveDateTime::parse_from_str("2999-12-31 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();
        let result = fetch_latest_tree_from_github(last_update).await.unwrap();
        assert!(result.is_none());
    }
}
