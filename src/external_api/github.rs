use chrono::{NaiveDate, Utc};

use crate::utils::{
    bin_parser::{BinDepositTree, BinEligibleTree},
    config::Settings,
    retry::with_retry,
};

#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Failed to deserialize data: {0}")]
    DeserializeError(String),
}

#[derive(Debug)]
pub struct BinTrees {
    pub bin_deposit_tree: BinDepositTree,
    pub bin_short_term_eligible_tree: BinEligibleTree,
    pub bin_long_term_eligible_tree: BinEligibleTree,
    pub latest_update: NaiveDate,
}

/// fetch content as bytes from a given URL
async fn fetch_bytes_content(url: &str) -> Result<Vec<u8>, GithubError> {
    let client = reqwest::Client::new();
    let response = with_retry(|| async {
        client
            .get(url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await
    })
    .await
    .map_err(|e| GithubError::NetworkError(e.to_string()))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| GithubError::DeserializeError(e.to_string()))?
        .to_vec();
    Ok(bytes)
}

pub async fn fetch_latest_tree_from_github(
    last_update: NaiveDate,
) -> Result<BinTrees, GithubError> {
    log::info!(
        "Fetching latest deposit and eligible trees from GitHub, last update: {}",
        last_update
    );
    let settings = Settings::load().unwrap();
    let directory_url = format!(
        "https://github.com/{}/raw/refs/heads/{}/{}",
        settings.api.tree_data_repository,
        settings.api.tree_data_branch,
        settings.api.tree_data_directory
    );

    log::info!("Fetching deposit tree");
    let deposit_tree_url = format!("{}/depositTree.txt", directory_url);
    let bin_deposit_tree_bytes = fetch_bytes_content(&deposit_tree_url).await?;
    let bin_deposit_tree: BinDepositTree =
        bincode::deserialize(&bin_deposit_tree_bytes).map_err(|e| {
            GithubError::DeserializeError(format!("failed to deserialize deposit tree: {}", e))
        })?;

    log::info!("Fetching short term eligible tree");
    let short_eligible_tree_url = format!("{}/eligibleTree-shortTerm.txt", directory_url);
    let bin_short_term_eligible_tree_bytes = fetch_bytes_content(&short_eligible_tree_url).await?;
    let bin_short_term_eligible_tree: BinEligibleTree =
        bincode::deserialize(&bin_short_term_eligible_tree_bytes).map_err(|e| {
            GithubError::DeserializeError(format!(
                "failed to deserialize short term eligible tree: {}",
                e
            ))
        })?;

    log::info!("Fetching long term eligible tree");
    let long_eligible_tree_url = format!("{}/eligibleTree-longTerm.txt", directory_url);
    let bin_long_term_eligible_tree_bytes = fetch_bytes_content(&long_eligible_tree_url).await?;
    let bin_long_term_eligible_tree: BinEligibleTree =
        bincode::deserialize(&bin_long_term_eligible_tree_bytes).map_err(|e| {
            GithubError::DeserializeError(format!(
                "failed to deserialize long term eligible tree: {}",
                e
            ))
        })?;

    let latest_update = Utc::now().naive_utc().date();
    Ok(BinTrees {
        bin_deposit_tree,
        bin_short_term_eligible_tree,
        bin_long_term_eligible_tree,
        latest_update,
    })
}

#[cfg(test)]
mod tests {
    use crate::utils::bin_parser::BinDepositTree;

    use super::*;

    #[tokio::test]
    async fn test_fetch_bytes_content() {
        let url = "https://github.com/InternetMaximalism/intmax2-v1-mining-mock/raw/refs/heads/main/base-sepolia-data/depositTree.txt";
        let bytes = fetch_bytes_content(url).await.unwrap();
        let _bin_deposit_tree: BinDepositTree = bincode::deserialize(&bytes).unwrap();
        dbg!(_bin_deposit_tree.block_number);
    }
}
